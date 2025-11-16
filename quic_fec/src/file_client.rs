//! File Transfer Client
//!
//! Complete client-side file transfer implementation

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::Instant;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use crate::connection::{QuicFecConnection, ConnectionConfig};
use crate::protocol::*;
use crate::scheduler::PacketPriority;

/// Transfer ID type
pub type TransferId = String;

/// Client transfer state
#[derive(Clone)]
pub struct ClientTransfer {
    pub transfer_id: TransferId,
    pub file_path: PathBuf,
    pub remote_path: String,
    pub total_size: u64,
    pub bytes_sent: u64,
    pub chunks_sent: HashSet<u64>,
    pub chunks_total: usize,
    pub status: TransferStatus,
    pub priority: PacketPriority,
    pub started_at: Instant,
    pub file_hash: [u8; 32],
    #[allow(dead_code)]
    pub progress_callback: Option<Arc<dyn Fn(ProgressUpdate) + Send + Sync>>,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Queued,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Progress update
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub transfer_id: TransferId,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed_mbps: f32,
    pub eta_seconds: Option<u64>,
    pub chunks_sent: usize,
    pub chunks_total: usize,
}

/// File transfer client
pub struct FileTransferClient {
    connection: QuicFecConnection,
    session_id: Option<String>,
    active_transfers: Arc<RwLock<HashMap<TransferId, ClientTransfer>>>,
    transfer_queue: Arc<RwLock<Vec<QueuedTransfer>>>,
    chunk_size: usize,
}

/// Queued transfer
#[derive(Debug, Clone)]
struct QueuedTransfer {
    transfer_id: TransferId,
    file_path: PathBuf,
    remote_path: String,
    priority: PacketPriority,
    queued_at: Instant,
}

impl FileTransferClient {
    /// Create new file transfer client
    pub async fn new(
        server_addr: std::net::SocketAddr,
        server_name: &str,
        connection_config: ConnectionConfig,
    ) -> Result<Self> {
        // Connect to server
        let connection = QuicFecConnection::connect(
            server_addr,
            server_name,
            connection_config,
        ).await
        .context("Failed to connect to server")?;

        Ok(Self {
            connection,
            session_id: None,
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            transfer_queue: Arc::new(RwLock::new(Vec::new())),
            chunk_size: 64 * 1024, // 64KB
        })
    }

    /// Perform 3-way handshake and authenticate
    pub async fn connect(
        &mut self,
        client_id: &str,
        auth_token: Option<&str>,
    ) -> Result<()> {
        // Step 1: Send connection request
        let connect_req = ClientMessage::Connect(ConnectRequest {
            client_id: client_id.to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            auth_token: auth_token.map(|s| s.to_string()),
            capabilities: ClientCapabilities {
                supports_resume: true,
                supports_parallel: true,
                supports_compression: true,
                max_chunk_size: 1024 * 1024, // 1MB
            },
        });

        self.send_message(&connect_req).await?;

        // Step 2: Receive connection accepted
        let response = self.receive_message().await?;
        
        match response {
            ServerMessage::ConnectionAccepted { session_id, .. } => {
                self.session_id = Some(session_id.clone());
                
                // Step 3: Send connection established
                let established = ClientMessage::ConnectionEstablished {
                    session_id: session_id.clone(),
                };
                self.send_message(&established).await?;

                println!("✅ Connected to server, session: {}", session_id);
                Ok(())
            }
            ServerMessage::ConnectionRejected(reason) => {
                Err(anyhow::anyhow!("Connection rejected: {}", reason))
            }
            _ => {
                Err(anyhow::anyhow!("Unexpected server response"))
            }
        }
    }

    /// Transfer a file
    pub async fn transfer_file(
        &self,
        file_path: &Path,
        remote_path: &str,
        priority: PacketPriority,
    ) -> Result<TransferId> {
        // Read file metadata
        let metadata = fs::metadata(file_path).await
            .context("Failed to read file metadata")?;
        let file_size = metadata.len();

        // Compute file hash
        let file_data = fs::read(file_path).await
            .context("Failed to read file")?;
        let file_hash = common::blake3_hash(&file_data);

        // Generate transfer ID
        let transfer_id = Uuid::new_v4().to_string();

        // Create transfer request
        let start_req = ClientMessage::StartTransfer(StartTransferRequest {
            transfer_id: transfer_id.clone(),
            file_name: file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string(),
            remote_path: remote_path.to_string(),
            file_size,
            file_hash: Some(file_hash),
            priority,
            resume_offset: None,
            preserve_metadata: true,
        });

        // Send start transfer request
        self.send_message(&start_req).await?;

        // Wait for transfer accepted
        let response = self.receive_message().await?;
        
        let chunk_size = match response {
            ServerMessage::TransferAccepted { chunk_size, .. } => chunk_size,
            ServerMessage::TransferRejected { reason, .. } => {
                return Err(anyhow::anyhow!("Transfer rejected: {}", reason));
            }
            _ => {
                return Err(anyhow::anyhow!("Unexpected server response"));
            }
        };

        // Create client transfer
        let transfer = ClientTransfer {
            transfer_id: transfer_id.clone(),
            file_path: file_path.to_path_buf(),
            remote_path: remote_path.to_string(),
            total_size: file_size,
            bytes_sent: 0,
            chunks_sent: HashSet::new(),
            chunks_total: ((file_size + chunk_size as u64 - 1) / chunk_size as u64) as usize,
            status: TransferStatus::InProgress,
            priority,
            started_at: Instant::now(),
            file_hash,
            progress_callback: None,
        };

        self.active_transfers.write().insert(transfer_id.clone(), transfer);

        // Send chunks synchronously (can be made async with proper connection sharing)
        self.send_file_chunks(&transfer_id, &file_data, chunk_size).await?;

        Ok(transfer_id)
    }

    /// Send file chunks
    async fn send_file_chunks(
        &self,
        transfer_id: &str,
        file_data: &[u8],
        chunk_size: usize,
    ) -> Result<()> {
        let total_chunks = (file_data.len() + chunk_size - 1) / chunk_size;

        for chunk_index in 0..total_chunks {
            let offset = chunk_index * chunk_size;
            let end = (offset + chunk_size).min(file_data.len());
            let chunk_data = &file_data[offset..end];
            let is_last = chunk_index == total_chunks - 1;

            // Compute chunk hash
            let chunk_hash = common::blake3_hash(chunk_data);

            // Create chunk message
            let chunk_msg = ClientMessage::SendChunk(ChunkData {
                transfer_id: transfer_id.to_string(),
                chunk_index: chunk_index as u64,
                offset: offset as u64,
                data: chunk_data.to_vec(),
                chunk_hash,
                is_last,
            });

            // Send chunk
            self.send_message(&chunk_msg).await?;

            // Update transfer state
            {
                let mut transfers = self.active_transfers.write();
                if let Some(transfer) = transfers.get_mut(transfer_id) {
                    transfer.chunks_sent.insert(chunk_index as u64);
                    transfer.bytes_sent += chunk_data.len() as u64;

                    // Update progress
                    if let Some(callback) = &transfer.progress_callback {
                        let progress = ProgressUpdate {
                            transfer_id: transfer_id.to_string(),
                            bytes_transferred: transfer.bytes_sent,
                            total_bytes: transfer.total_size,
                            percentage: (transfer.bytes_sent as f32 / transfer.total_size as f32) * 100.0,
                            speed_mbps: self.calculate_speed(transfer),
                            eta_seconds: self.calculate_eta(transfer),
                            chunks_sent: transfer.chunks_sent.len(),
                            chunks_total: transfer.chunks_total,
                        };
                        callback(progress);
                    }
                }
            }

            // Wait for chunk acknowledgment
            let response = self.receive_message().await?;
            match response {
                ServerMessage::ChunkReceived { .. } => {
                    // Chunk received, continue
                }
                ServerMessage::TransferComplete { .. } => {
                    // Transfer complete
                    if let Some(transfer) = self.active_transfers.write().get_mut(transfer_id) {
                        transfer.status = TransferStatus::Completed;
                    }
                    break;
                }
                ServerMessage::TransferError { error, .. } => {
                    return Err(anyhow::anyhow!("Transfer error: {}", error));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Calculate transfer speed
    fn calculate_speed(&self, transfer: &ClientTransfer) -> f32 {
        let elapsed = transfer.started_at.elapsed().as_secs_f32();
        if elapsed > 0.0 {
            (transfer.bytes_sent as f32 / (1024.0 * 1024.0)) / elapsed
        } else {
            0.0
        }
    }

    /// Calculate ETA
    fn calculate_eta(&self, transfer: &ClientTransfer) -> Option<u64> {
        let speed = self.calculate_speed(transfer);
        if speed > 0.0 {
            let remaining = transfer.total_size - transfer.bytes_sent;
            Some((remaining as f32 / (1024.0 * 1024.0) / speed) as u64)
        } else {
            None
        }
    }

    /// Set progress callback
    pub fn set_progress_callback<F>(&self, transfer_id: &str, callback: F) -> Result<()>
    where
        F: Fn(ProgressUpdate) + Send + Sync + 'static,
    {
        let mut transfers = self.active_transfers.write();
        if let Some(transfer) = transfers.get_mut(transfer_id) {
            transfer.progress_callback = Some(Arc::new(callback));
        }
        Ok(())
    }

    /// Get transfer status
    pub fn get_transfer_status(&self, transfer_id: &str) -> Option<ClientTransfer> {
        self.active_transfers.read().get(transfer_id).cloned()
    }

    /// Send message to server
    async fn send_message(&self, msg: &ClientMessage) -> Result<()> {
        let data = serde_json::to_vec(msg)?;
        self.connection.send(bytes::Bytes::from(data)).await?;
        Ok(())
    }

    /// Receive message from server
    async fn receive_message(&self) -> Result<ServerMessage> {
        let data = self.connection.recv().await?
            .ok_or_else(|| anyhow::anyhow!("No data received"))?;
        let msg: ServerMessage = serde_json::from_slice(&data)?;
        Ok(msg)
    }
}

