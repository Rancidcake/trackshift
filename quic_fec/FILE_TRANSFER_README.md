# QUIC-FEC File Transfer System

Complete end-to-end file transfer system with ECDHE handshake, 3-way connection establishment, and full client-server implementation.

## Features

### ✅ Implemented

- **TLS 1.3 with ECDHE**: Automatic ECDHE key exchange via TLS 1.3
- **3-Way Handshake**: 
  1. Client sends connection request
  2. Server sends connection accepted
  3. Client sends connection established
- **File Transfer**: Complete file chunking, transfer, and reassembly
- **Session Management**: Track client sessions and active transfers
- **Authentication**: Token-based authentication with user permissions
- **Progress Tracking**: Real-time transfer progress with callbacks
- **File Integrity**: Blake3 hash verification
- **FEC Support**: Forward Error Correction for unreliable networks
- **Resume Capability**: Track chunks for potential resume (foundation ready)

## Architecture

### Server Side

```
QuicFecServer
├── ConnectionManager (handles multiple clients)
├── FileTransferHandler (file operations)
├── SessionManager (session tracking)
├── AuthManager (authentication)
└── ChunkStorage (chunk storage & reassembly)
```

### Client Side

```
FileTransferClient
├── QuicFecConnection (QUIC transport)
├── TransferQueue (queue management)
├── ProgressTracker (progress updates)
└── ResumeDatabase (resume capability - ready)
```

## Protocol

### 3-Way Handshake

1. **Client → Server**: `ConnectRequest`
   ```json
   {
     "client_id": "client-1",
     "client_version": "0.1.0",
     "auth_token": "admin_token",
     "capabilities": {...}
   }
   ```

2. **Server → Client**: `ConnectionAccepted`
   ```json
   {
     "session_id": "uuid",
     "server_capabilities": {...}
   }
   ```

3. **Client → Server**: `ConnectionEstablished`
   ```json
   {
     "session_id": "uuid"
   }
   ```

### File Transfer Messages

- `StartTransfer`: Initiate file transfer
- `SendChunk`: Send file chunk with hash
- `ChunkReceived`: Acknowledgment
- `TransferProgress`: Progress updates
- `TransferComplete`: Transfer finished
- `TransferError`: Error occurred

## Usage

### 1. Generate Certificate

```bash
cd quic_fec/examples
chmod +x generate_cert.sh
./generate_cert.sh
```

This creates:
- `server.crt` - Certificate
- `server.key` - Private key

### 2. Run Server

```bash
cargo run --example server -- 127.0.0.1:8080 ./server_storage
```

Arguments:
- `127.0.0.1:8080` - Server address
- `./server_storage` - Storage directory (optional, defaults to `server_storage`)

### 3. Run Client

```bash
cargo run --example client -- 127.0.0.1:8080 ./test.txt /uploads/test.txt
```

Arguments:
- `127.0.0.1:8080` - Server address
- `./test.txt` - Local file to transfer
- `/uploads/test.txt` - Remote path (optional, defaults to filename)

## Code Examples

### Server

```rust
use quic_fec::QuicFecServer;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

// Load certificate
let (cert, key) = load_certificate()?;

// Create server
let server = QuicFecServer::new(
    "127.0.0.1:8080".parse()?,
    cert,
    key,
    PathBuf::from("storage"),
)?;

// Run server
server.run().await?;
```

### Client

```rust
use quic_fec::{FileTransferClient, ConnectionConfig, PacketPriority};

// Create client
let mut client = FileTransferClient::new(
    "127.0.0.1:8080".parse()?,
    "localhost",
    ConnectionConfig::default(),
).await?;

// Connect and authenticate
client.connect("client-1", Some("admin_token")).await?;

// Transfer file
let transfer_id = client.transfer_file(
    &PathBuf::from("test.txt"),
    "/uploads/test.txt",
    PacketPriority::High,
).await?;

// Set progress callback
client.set_progress_callback(&transfer_id, |progress| {
    println!("Progress: {:.2}%", progress.percentage);
})?;
```

## Security

### TLS 1.3 with ECDHE

- **ECDHE Key Exchange**: Automatic in TLS 1.3
- **Perfect Forward Secrecy**: Each connection uses ephemeral keys
- **Certificate Validation**: Server certificate validation (can be configured)

### Authentication

- **Token-based**: Simple token authentication
- **User Permissions**: ReadOnly, ReadWrite, Admin
- **Rate Limiting**: Per-user rate limits (foundation ready)

## File Transfer Process

1. **Client initiates transfer**:
   - Reads file
   - Computes Blake3 hash
   - Sends `StartTransfer` request

2. **Server accepts**:
   - Creates transfer record
   - Allocates storage
   - Sends `TransferAccepted`

3. **Client sends chunks**:
   - Chunks file (64KB default)
   - Computes chunk hash
   - Sends `SendChunk` messages
   - Waits for acknowledgment

4. **Server stores chunks**:
   - Validates chunk hash
   - Stores chunk to temp file
   - Sends `ChunkReceived` acknowledgment
   - Tracks progress

5. **Transfer completion**:
   - Server reassembles file from chunks
   - Verifies file integrity
   - Sends `TransferComplete`

## Configuration

### Connection Config

```rust
ConnectionConfig {
    fec_config: FecConfig::for_file_transfer(),
    handover_strategy: HandoverStrategy::Smooth,
    initial_path: NetworkPath::WiFi,
    enable_fec: true,
    max_retransmissions: 3,
}
```

### FEC Config

```rust
FecConfig {
    data_shards: 4,
    parity_shards: 2,
    max_shard_size: 1400,
}
```

## Performance

- **Chunk Size**: 64KB (configurable)
- **Concurrent Transfers**: Multiple transfers per session
- **Progress Updates**: Real-time progress callbacks
- **Network Efficiency**: FEC for unreliable networks

## Future Enhancements

- [ ] Resume capability (foundation ready)
- [ ] Directory transfer
- [ ] Parallel chunk sending
- [ ] Bandwidth throttling
- [ ] Transfer queue management
- [ ] Web UI for server management

## Troubleshooting

### Certificate Issues

If you get certificate errors:
1. Generate certificate: `./generate_cert.sh`
2. Ensure `server.crt` and `server.key` exist
3. For production, use proper CA-signed certificates

### Connection Issues

- Check firewall settings
- Ensure server is running
- Verify server address and port

### Transfer Failures

- Check file permissions
- Verify storage directory exists
- Check disk space
- Review server logs

## License

See main project LICENSE file.

