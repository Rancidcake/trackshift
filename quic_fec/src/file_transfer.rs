//! File Transfer Handler (Server-side)
//!
//! Handles file storage, chunk management, and reassembly

use anyhow::{Result, Context};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::Instant;
use tokio::fs;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use crate::scheduler::PacketPriority;

/// File transfer request
#[derive(Debug, Clone)]
pub struct FileTransferRequest {
    pub transfer_id: String,
    pub file_path: String,
    pub file_size: u64,
    pub file_hash: Option<[u8; 32]>,
    pub priority: PacketPriority,
    pub resume_offset: Option<u64>,
}

/// Active transfer state
#[derive(Debug, Clone)]
pub struct ActiveTransfer {
    pub transfer_id: String,
    pub file_path: PathBuf,
    pub remote_path: String,
    pub total_size: u64,
    pub bytes_received: u64,
    pub chunks_received: HashSet<u64>,
    pub chunks_total: usize,
    pub status: TransferStatus,
    pub started_at: Instant,
    pub priority: PacketPriority,
    pub file_hash: Option<[u8; 32]>,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Transfer progress
#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub transfer_id: String,
    pub bytes_received: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub chunks_received: usize,
    pub chunks_total: usize,
    pub is_complete: bool,
    pub speed_mbps: f32,
}

/// File transfer handler
pub struct FileTransferHandler {
    storage_path: PathBuf,
    temp_path: PathBuf,
    active_transfers: Arc<RwLock<HashMap<String, ActiveTransfer>>>,
    chunk_storage: Arc<RwLock<HashMap<String, HashMap<u64, PathBuf>>>>, // transfer_id -> chunk_index -> path
}

impl FileTransferHandler {
    /// Create new file transfer handler
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        let temp_path = storage_path.join("temp");
        
        // Create directories
        std::fs::create_dir_all(&storage_path)
            .context("Failed to create storage directory")?;
        std::fs::create_dir_all(&temp_path)
            .context("Failed to create temp directory")?;

        Ok(Self {
            storage_path,
            temp_path,
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            chunk_storage: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start a new file transfer
    pub async fn start_transfer(&self, request: FileTransferRequest) -> Result<String> {
        let transfer_id = request.transfer_id.clone();
        
        // Calculate number of chunks (64KB each)
        let chunk_size = 64 * 1024;
        let chunks_total = ((request.file_size + chunk_size as u64 - 1) / chunk_size as u64) as usize;

        // Create file path
        let file_path = self.storage_path.join(&request.file_path);
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create file directory")?;
        }

        // Create active transfer
        let transfer = ActiveTransfer {
            transfer_id: transfer_id.clone(),
            file_path: file_path.clone(),
            remote_path: request.file_path.clone(),
            total_size: request.file_size,
            bytes_received: request.resume_offset.unwrap_or(0),
            chunks_received: HashSet::new(),
            chunks_total,
            status: TransferStatus::InProgress,
            started_at: Instant::now(),
            priority: request.priority,
            file_hash: request.file_hash,
        };

        self.active_transfers.write().insert(transfer_id.clone(), transfer);

        // Initialize chunk storage
        self.chunk_storage.write().insert(transfer_id.clone(), HashMap::new());

        Ok(transfer_id)
    }

    /// Store a received chunk
    pub async fn store_chunk(
        &self,
        transfer_id: &str,
        chunk_index: u64,
        chunk_data: &[u8],
    ) -> Result<()> {
        // Check if already received (drop lock before await)
        {
            let transfers = self.active_transfers.read();
            let transfer = transfers.get(transfer_id)
                .ok_or_else(|| anyhow::anyhow!("Transfer not found: {}", transfer_id))?;
            if transfer.chunks_received.contains(&chunk_index) {
                return Ok(()); // Already have this chunk
            }
        }

        // Store chunk to temp file (no lock held during async operations)
        let chunk_path = self.temp_path.join(format!("{}_{}.chunk", transfer_id, chunk_index));
        let mut file = fs::File::create(&chunk_path).await
            .context("Failed to create chunk file")?;
        file.write_all(chunk_data).await
            .context("Failed to write chunk data")?;
        file.sync_all().await?;

        // Update transfer state (re-acquire lock after async operations)
        {
            let mut transfers = self.active_transfers.write();
            let transfer = transfers.get_mut(transfer_id)
                .ok_or_else(|| anyhow::anyhow!("Transfer not found: {}", transfer_id))?;
            transfer.chunks_received.insert(chunk_index);
            transfer.bytes_received += chunk_data.len() as u64;
        }

        // Update chunk storage map
        {
            let mut chunk_storage = self.chunk_storage.write();
            chunk_storage
                .entry(transfer_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(chunk_index, chunk_path);
        }

        Ok(())
    }

    /// Get transfer progress
    pub async fn get_progress(&self, transfer_id: &str) -> Result<Option<TransferProgress>> {
        let transfers = self.active_transfers.read();
        let transfer = match transfers.get(transfer_id) {
            Some(t) => t,
            None => return Ok(None),
        };

        let elapsed = transfer.started_at.elapsed().as_secs_f32();
        let speed_mbps = if elapsed > 0.0 {
            (transfer.bytes_received as f32 / (1024.0 * 1024.0)) / elapsed
        } else {
            0.0
        };

        let percentage = if transfer.total_size > 0 {
            (transfer.bytes_received as f32 / transfer.total_size as f32) * 100.0
        } else {
            0.0
        };

        let is_complete = transfer.chunks_received.len() >= transfer.chunks_total;

        Ok(Some(TransferProgress {
            transfer_id: transfer_id.to_string(),
            bytes_received: transfer.bytes_received,
            total_bytes: transfer.total_size,
            percentage,
            chunks_received: transfer.chunks_received.len(),
            chunks_total: transfer.chunks_total,
            is_complete,
            speed_mbps,
        }))
    }

    /// Reassemble file from chunks
    pub async fn reassemble_file(&self, transfer_id: &str) -> Result<PathBuf> {
        // Extract needed data (drop locks before await)
        let (file_path, total_chunks, chunk_paths) = {
            let transfers = self.active_transfers.read();
            let transfer = transfers.get(transfer_id)
                .ok_or_else(|| anyhow::anyhow!("Transfer not found: {}", transfer_id))?;

            let chunk_storage = self.chunk_storage.read();
            let chunks = chunk_storage.get(transfer_id)
                .ok_or_else(|| anyhow::anyhow!("Chunk storage not found: {}", transfer_id))?;

            // Clone paths we need
            let mut paths = Vec::new();
            for chunk_index in 0..transfer.chunks_total {
                if let Some(chunk_path) = chunks.get(&(chunk_index as u64)) {
                    paths.push(chunk_path.clone());
                } else {
                    return Err(anyhow::anyhow!("Missing chunk: {}", chunk_index));
                }
            }

            (transfer.file_path.clone(), transfer.chunks_total, paths)
        };

        // Create output file (no locks held)
        let mut output_file = fs::File::create(&file_path).await
            .context("Failed to create output file")?;

        // Write chunks in order
        for chunk_path in chunk_paths {
            // Read chunk
            let mut chunk_file = fs::File::open(&chunk_path).await
                .context("Failed to open chunk file")?;
            let mut chunk_data = Vec::new();
            chunk_file.read_to_end(&mut chunk_data).await
                .context("Failed to read chunk data")?;

            // Write to output file
            output_file.write_all(&chunk_data).await
                .context("Failed to write chunk to output file")?;
        }

        output_file.sync_all().await?;

        // Update transfer status (re-acquire lock)
        {
            let mut transfers = self.active_transfers.write();
            if let Some(transfer) = transfers.get_mut(transfer_id) {
                transfer.status = TransferStatus::Completed;
            }
        }

        Ok(file_path)
    }

    /// Verify file integrity
    pub async fn verify_file(&self, transfer_id: &str) -> Result<bool> {
        // Extract needed data (drop lock before await)
        let (file_path, expected_hash) = {
            let transfers = self.active_transfers.read();
            let transfer = transfers.get(transfer_id)
                .ok_or_else(|| anyhow::anyhow!("Transfer not found: {}", transfer_id))?;
            (transfer.file_path.clone(), transfer.file_hash)
        };

        if let Some(expected_hash) = expected_hash {
            // Read file and compute hash (no lock held)
            let file_data = fs::read(&file_path).await
                .context("Failed to read file for verification")?;
            
            let computed_hash = common::blake3_hash(&file_data);
            
            Ok(computed_hash == expected_hash)
        } else {
            // No hash provided, assume valid
            Ok(true)
        }
    }

    /// Cleanup incomplete transfers
    pub async fn cleanup_incomplete(&self, older_than_seconds: u64) -> Result<()> {
        let cutoff = Instant::now() - std::time::Duration::from_secs(older_than_seconds);
        
        let mut transfers = self.active_transfers.write();
        let mut to_remove = Vec::new();

        for (transfer_id, transfer) in transfers.iter() {
            if transfer.started_at < cutoff && transfer.status != TransferStatus::Completed {
                to_remove.push(transfer_id.clone());
            }
        }

        for transfer_id in to_remove {
            // Remove chunks
            let mut chunk_storage = self.chunk_storage.write();
            if let Some(chunks) = chunk_storage.remove(&transfer_id) {
                for chunk_path in chunks.values() {
                    let _ = fs::remove_file(chunk_path).await;
                }
            }

            // Remove transfer
            transfers.remove(&transfer_id);
        }

        Ok(())
    }
}

