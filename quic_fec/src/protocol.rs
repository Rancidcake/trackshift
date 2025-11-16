//! File Transfer Protocol
//!
//! Defines the message protocol between client and server

use serde::{Serialize, Deserialize};
use crate::scheduler::PacketPriority;

/// Client-to-Server messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Initial connection request (3-way handshake step 1)
    Connect(ConnectRequest),
    
    /// Start file transfer
    StartTransfer(StartTransferRequest),
    
    /// Send file chunk
    SendChunk(ChunkData),
    
    /// Pause transfer
    PauseTransfer {
        transfer_id: String,
    },
    
    /// Resume transfer
    ResumeTransfer {
        transfer_id: String,
        resume_offset: u64,
    },
    
    /// Cancel transfer
    CancelTransfer {
        transfer_id: String,
    },
    
    /// Query transfer status
    QueryStatus(String), // transfer_id
    
    /// List files in directory
    ListFiles {
        path: String,
    },
    
    /// Connection established (3-way handshake step 3)
    ConnectionEstablished {
        session_id: String,
    },
}

/// Server-to-Client messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Connection accepted (3-way handshake step 2)
    ConnectionAccepted {
        session_id: String,
        server_capabilities: ServerCapabilities,
    },
    
    /// Connection rejected
    ConnectionRejected(String), // reason
    
    /// Transfer accepted
    TransferAccepted {
        transfer_id: String,
        chunk_size: usize,
    },
    
    /// Transfer rejected
    TransferRejected {
        transfer_id: String,
        reason: String,
    },
    
    /// Chunk received acknowledgment
    ChunkReceived {
        transfer_id: String,
        chunk_index: u64,
    },
    
    /// Transfer progress update
    TransferProgress {
        transfer_id: String,
        bytes_received: u64,
        total_bytes: u64,
        percentage: f32,
    },
    
    /// Transfer complete
    TransferComplete {
        transfer_id: String,
        file_path: String,
        file_size: u64,
    },
    
    /// Transfer error
    TransferError {
        transfer_id: String,
        error: String,
    },
    
    /// File list response
    FileList {
        files: Vec<FileEntry>,
    },
}

/// Connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub client_id: String,
    pub client_version: String,
    pub auth_token: Option<String>,
    pub capabilities: ClientCapabilities,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub supports_resume: bool,
    pub supports_parallel: bool,
    pub supports_compression: bool,
    pub max_chunk_size: usize,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub max_file_size: u64,
    pub max_concurrent_transfers: usize,
    pub supported_features: Vec<String>,
}

/// Start transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTransferRequest {
    pub transfer_id: String,
    pub file_name: String,
    pub remote_path: String,
    pub file_size: u64,
    pub file_hash: Option<[u8; 32]>, // Blake3 hash
    pub priority: PacketPriority,
    pub resume_offset: Option<u64>,
    pub preserve_metadata: bool,
}

/// Chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub transfer_id: String,
    pub chunk_index: u64,
    pub offset: u64,
    pub data: Vec<u8>,
    pub chunk_hash: [u8; 32], // Blake3 hash of chunk
    pub is_last: bool,
}

/// File entry (for directory listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub modified: Option<String>, // ISO 8601 timestamp
    pub permissions: Option<u32>,
}

