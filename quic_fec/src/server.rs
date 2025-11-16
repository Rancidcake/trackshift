//! Complete QUIC-FEC Server Implementation
//!
//! Provides a full-featured server with:
//! - TLS 1.3 with ECDHE key exchange
//! - Multiple concurrent connections
//! - File transfer handling
//! - Session management
//! - Authentication

use anyhow::{Result, Context};
use quinn::{Endpoint, ServerConfig};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::connection::ConnectionConfig;
use crate::file_transfer::{FileTransferHandler, FileTransferRequest};
use crate::session::{SessionManager, Session};
use crate::auth::AuthManager;

/// QUIC-FEC Server
pub struct QuicFecServer {
    endpoint: Endpoint,
    config: ServerConfig,
    file_handler: Arc<FileTransferHandler>,
    session_manager: Arc<SessionManager>,
    auth_manager: Arc<AuthManager>,
    connection_config: ConnectionConfig,
    active_connections: Arc<RwLock<HashMap<u64, quinn::Connection>>>,
    connection_counter: Arc<RwLock<u64>>,
}

impl QuicFecServer {
    /// Create a new QUIC-FEC server
    ///
    /// # Arguments
    /// * `addr` - Server bind address
    /// * `cert` - TLS certificate (DER format)
    /// * `key` - Private key (DER format)
    /// * `storage_path` - Path for file storage
    pub fn new(
        addr: SocketAddr,
        cert: CertificateDer<'static>,
        key: PrivateKeyDer<'static>,
        storage_path: std::path::PathBuf,
    ) -> Result<Self> {
        // Create TLS server config with ECDHE
        // TLS 1.3 automatically uses ECDHE for key exchange
        let tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .context("Failed to create TLS config")?;

        // Create QUIC server config using quinn's crypto wrapper
        // For quinn 0.11, we use the rustls crypto provider
        let crypto = quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)
            .context("Failed to create QUIC crypto config")?;
        let mut server_config = ServerConfig::with_crypto(Arc::new(crypto));
        
        // Configure transport parameters
        let mut transport = quinn::TransportConfig::default();
        transport.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into()?));
        transport.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
        server_config.transport = Arc::new(transport);

        // Create QUIC endpoint
        let endpoint = Endpoint::server(server_config.clone(), addr)
            .context("Failed to create QUIC endpoint")?;

        // Create file transfer handler
        let file_handler = Arc::new(
            FileTransferHandler::new(storage_path)
                .context("Failed to create file transfer handler")?
        );

        // Create session manager
        let session_manager = Arc::new(SessionManager::new());

        // Create auth manager
        let auth_manager = Arc::new(AuthManager::new());

        // Default connection config
        let connection_config = ConnectionConfig::default();

        Ok(Self {
            endpoint,
            config: server_config,
            file_handler,
            session_manager,
            auth_manager,
            connection_config,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            connection_counter: Arc::new(RwLock::new(0)),
        })
    }

    /// Run the server (accepts connections)
    pub async fn run(&self) -> Result<()> {
        println!("🚀 QUIC-FEC Server listening on {}", self.endpoint.local_addr()?);
        println!("📡 Waiting for connections...");

        // Accept connections in a loop
        while let Some(conn) = self.endpoint.accept().await {
            let connection = conn.await?;
            
            // Generate connection ID
            let conn_id = {
                let mut counter = self.connection_counter.write();
                *counter += 1;
                *counter
            };

            println!("✅ New connection: {} (ID: {})", connection.remote_address(), conn_id);

            // Clone shared resources
            let file_handler = Arc::clone(&self.file_handler);
            let session_manager = Arc::clone(&self.session_manager);
            let auth_manager = Arc::clone(&self.auth_manager);
            let connection_config = self.connection_config.clone();
            let active_connections = Arc::clone(&self.active_connections);

            // Handle connection in background task
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(
                    connection,
                    conn_id,
                    file_handler,
                    session_manager,
                    auth_manager,
                    connection_config,
                    active_connections,
                ).await {
                    eprintln!("❌ Connection {} error: {}", conn_id, e);
                }
            });
        }

        Ok(())
    }

    /// Handle a single connection
    async fn handle_connection(
        connection: quinn::Connection,
        conn_id: u64,
        file_handler: Arc<FileTransferHandler>,
        session_manager: Arc<SessionManager>,
        auth_manager: Arc<AuthManager>,
        connection_config: ConnectionConfig,
        active_connections: Arc<RwLock<HashMap<u64, quinn::Connection>>>,
    ) -> Result<()> {
        // Store connection
        active_connections.write().insert(conn_id, connection.clone());

        // Perform 3-way handshake (QUIC style)
        // 1. Client sends connection request
        // 2. Server sends connection accept
        // 3. Client sends connection established

        // Wait for client handshake message
        let mut recv_stream = connection.accept_uni().await?;
        let mut handshake_buf = Vec::new();
        let mut chunk = vec![0u8; 1024];

        loop {
            match recv_stream.read(&mut chunk).await {
                Ok(Some(0)) => break,
                Ok(Some(n)) if n > 0 && n <= chunk.len() => {
                    handshake_buf.extend_from_slice(&chunk[..n]);
                }
                Ok(Some(_)) | Ok(None) => break,
                Err(_) => break,
            }
        }

        // Parse handshake message
        let handshake_msg: crate::protocol::ClientMessage = 
            serde_json::from_slice(&handshake_buf)
                .context("Failed to parse handshake message")?;

        // Handle authentication
        let session_id = match handshake_msg {
            crate::protocol::ClientMessage::Connect(req) => {
                // Authenticate client
                let auth_result = auth_manager.authenticate(req.auth_token.as_ref()).await?;
                
                if !auth_result.is_authenticated {
                    // Send rejection
                    let reject_msg = crate::protocol::ServerMessage::ConnectionRejected(
                        "Authentication failed".to_string()
                    );
                    let mut send_stream = connection.open_uni().await?;
                    send_stream.write_all(&serde_json::to_vec(&reject_msg)?).await?;
                    send_stream.finish()?;
                    return Err(anyhow::anyhow!("Authentication failed"));
                }

                // Create session
                let session = Session::new(
                    conn_id,
                    req.client_id,
                    auth_result.user_id,
                );
                let session_id = session.session_id.clone();
                session_manager.create_session(session).await?;

                // Send connection accepted
                let accept_msg = crate::protocol::ServerMessage::ConnectionAccepted {
                    session_id: session_id.clone(),
                    server_capabilities: crate::protocol::ServerCapabilities {
                        max_file_size: 10 * 1024 * 1024 * 1024, // 10GB
                        max_concurrent_transfers: 10,
                        supported_features: vec![
                            "resume".to_string(),
                            "parallel".to_string(),
                            "compression".to_string(),
                        ],
                    },
                };
                let mut send_stream = connection.open_uni().await?;
                send_stream.write_all(&serde_json::to_vec(&accept_msg)?).await?;
                send_stream.finish()?;

                session_id
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid handshake message"));
            }
        };

        println!("✅ Connection {} authenticated, session: {}", conn_id, session_id);

        // Main connection loop - handle file transfer requests
        loop {
            // Accept incoming streams
            let mut recv_stream = match connection.accept_uni().await {
                Ok(stream) => stream,
                Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                    println!("Connection {} closed by client", conn_id);
                    break;
                }
                Err(e) => {
                    eprintln!("Stream accept error: {}", e);
                    break;
                }
            };

            // Read message
            let mut message_buf = Vec::new();
            let mut chunk = vec![0u8; 8192];

            loop {
                match recv_stream.read(&mut chunk).await {
                    Ok(Some(0)) => break,
                    Ok(Some(n)) if n > 0 && n <= chunk.len() => {
                        message_buf.extend_from_slice(&chunk[..n]);
                    }
                    Ok(Some(_)) | Ok(None) => break,
                    Err(_) => break,
                }
            }

            if message_buf.is_empty() {
                continue;
            }

            // Parse and handle message
            match serde_json::from_slice::<crate::protocol::ClientMessage>(&message_buf) {
                Ok(msg) => {
                    if let Err(e) = Self::handle_message(
                        &msg,
                        &file_handler,
                        &session_manager,
                        &connection,
                        &session_id,
                    ).await {
                        eprintln!("Error handling message: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                }
            }
        }

        // Cleanup
        active_connections.write().remove(&conn_id);
        session_manager.remove_session(&session_id).await?;

        Ok(())
    }

    /// Handle client message
    async fn handle_message(
        msg: &crate::protocol::ClientMessage,
        file_handler: &Arc<FileTransferHandler>,
        session_manager: &Arc<SessionManager>,
        connection: &quinn::Connection,
        session_id: &str,
    ) -> Result<()> {
        match msg {
            crate::protocol::ClientMessage::StartTransfer(req) => {
                // Create transfer request
                let transfer_req = FileTransferRequest {
                    transfer_id: req.transfer_id.clone(),
                    file_path: req.remote_path.clone(),
                    file_size: req.file_size,
                    file_hash: req.file_hash,
                    priority: req.priority,
                    resume_offset: req.resume_offset,
                };

                // Start transfer
                let transfer_id = file_handler.start_transfer(transfer_req).await?;

                // Send transfer accepted
                let response = crate::protocol::ServerMessage::TransferAccepted {
                    transfer_id: transfer_id.clone(),
                    chunk_size: 64 * 1024, // 64KB chunks
                };
                Self::send_message(connection, &response).await?;

                // Update session
                session_manager.add_transfer(session_id, &transfer_id).await?;
            }

            crate::protocol::ClientMessage::SendChunk(chunk_data) => {
                // Store chunk
                file_handler.store_chunk(
                    &chunk_data.transfer_id,
                    chunk_data.chunk_index,
                    &chunk_data.data,
                ).await?;

                // Send chunk received acknowledgment
                let response = crate::protocol::ServerMessage::ChunkReceived {
                    transfer_id: chunk_data.transfer_id.clone(),
                    chunk_index: chunk_data.chunk_index,
                };
                Self::send_message(connection, &response).await?;

                // Check if transfer is complete
                if let Some(progress) = file_handler.get_progress(&chunk_data.transfer_id).await
                    .context("Failed to get transfer progress")? {
                    if progress.is_complete {
                        // Reassemble file
                        let file_path = file_handler.reassemble_file(&chunk_data.transfer_id).await?;

                        // Verify integrity
                        let verified = file_handler.verify_file(&chunk_data.transfer_id).await?;

                        let response = if verified {
                            crate::protocol::ServerMessage::TransferComplete {
                                transfer_id: chunk_data.transfer_id.clone(),
                                file_path: file_path.to_string_lossy().to_string(),
                                file_size: progress.total_bytes,
                            }
                        } else {
                            crate::protocol::ServerMessage::TransferError {
                                transfer_id: chunk_data.transfer_id.clone(),
                                error: "File integrity verification failed".to_string(),
                            }
                        };
                        Self::send_message(connection, &response).await?;
                    } else {
                        // Send progress update
                        let response = crate::protocol::ServerMessage::TransferProgress {
                            transfer_id: chunk_data.transfer_id.clone(),
                            bytes_received: progress.bytes_received,
                            total_bytes: progress.total_bytes,
                            percentage: progress.percentage,
                        };
                        Self::send_message(connection, &response).await?;
                    }
                }
            }

            crate::protocol::ClientMessage::QueryStatus(transfer_id) => {
                if let Some(progress) = file_handler.get_progress(transfer_id).await
                    .context("Failed to get transfer progress")? {
                    let response = crate::protocol::ServerMessage::TransferProgress {
                        transfer_id: transfer_id.clone(),
                        bytes_received: progress.bytes_received,
                        total_bytes: progress.total_bytes,
                        percentage: progress.percentage,
                    };
                    Self::send_message(connection, &response).await?;
                }
            }

            _ => {
                // Handle other message types
            }
        }

        Ok(())
    }

    /// Send message to client
    async fn send_message(
        connection: &quinn::Connection,
        msg: &crate::protocol::ServerMessage,
    ) -> Result<()> {
        let mut send_stream = connection.open_uni().await?;
        let data = serde_json::to_vec(msg)?;
        send_stream.write_all(&data).await?;
        send_stream.finish()?;
        Ok(())
    }

    /// Get server address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.endpoint.local_addr()
            .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))
    }

    /// Shutdown server
    pub fn shutdown(&self) {
        self.endpoint.close(0u32.into(), b"Server shutdown");
    }
}

