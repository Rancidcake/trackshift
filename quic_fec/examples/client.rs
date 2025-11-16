//! QUIC-FEC File Transfer Client Example
//!
//! This example demonstrates a complete file transfer client with:
//! - TLS 1.3 with ECDHE key exchange (automatic in TLS 1.3)
//! - 3-way handshake for connection establishment
//! - File transfer with progress tracking
//! - Resume capability

use anyhow::{Result, Context};
use quic_fec::{FileTransferClient, ConnectionConfig, PacketPriority, HandoverStrategy, NetworkPath};
use std::net::SocketAddr;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("📤 QUIC-FEC File Transfer Client");
    println!("================================\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <server_addr> <file_path> [remote_path]", args[0]);
        eprintln!("Example: {} 127.0.0.1:8080 ./test.txt /uploads/test.txt", args[0]);
        std::process::exit(1);
    }

    let server_addr: SocketAddr = args[1]
        .parse()
        .context("Invalid server address")?;

    let file_path = PathBuf::from(&args[2]);
    let remote_path = args
        .get(3)
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
        })
        .to_string();

    println!("📡 Server: {}", server_addr);
    println!("📁 Local file: {}", file_path.display());
    println!("📤 Remote path: {}", remote_path);
    println!();

    // Check if file exists
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    // Create connection config
    let connection_config = ConnectionConfig {
        fec_config: quic_fec::FecConfig::for_file_transfer(),
        handover_strategy: HandoverStrategy::Smooth,
        initial_path: NetworkPath::WiFi,
        enable_fec: true,
        max_retransmissions: 3,
    };

    // Create client
    let mut client = FileTransferClient::new(
        server_addr,
        "localhost", // Server name for TLS
        connection_config,
    )
    .await
    .context("Failed to create client")?;

    println!("✅ Client created");
    println!();

    // Connect and authenticate (3-way handshake)
    println!("🔐 Connecting to server...");
    client
        .connect("client-1", Some("admin_token")) // Use admin token for testing
        .await
        .context("Failed to connect to server")?;

    println!("✅ Connected and authenticated");
    println!();

    // Set up progress callback
    let progress_callback = |progress: quic_fec::ProgressUpdate| {
        println!(
            "📊 Progress: {:.2}% ({:.2} MB/s) - {}/{} chunks",
            progress.percentage,
            progress.speed_mbps,
            progress.chunks_sent,
            progress.chunks_total
        );
        if let Some(eta) = progress.eta_seconds {
            println!("   ETA: {} seconds", eta);
        }
    };

    // Start file transfer
    println!("📤 Starting file transfer...");
    let transfer_id = client
        .transfer_file(&file_path, &remote_path, PacketPriority::High)
        .await
        .context("Failed to start transfer")?;

    // Set progress callback
    client
        .set_progress_callback(&transfer_id, progress_callback)
        .context("Failed to set progress callback")?;

    // Wait for transfer to complete
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        if let Some(transfer) = client.get_transfer_status(&transfer_id) {
            match transfer.status {
                quic_fec::TransferStatus::Completed => {
                    println!("\n✅ Transfer completed successfully!");
                    println!("   Transfer ID: {}", transfer_id);
                    println!("   File size: {} bytes", transfer.total_size);
                    break;
                }
                quic_fec::TransferStatus::Failed => {
                    eprintln!("\n❌ Transfer failed!");
                    std::process::exit(1);
                }
                quic_fec::TransferStatus::Cancelled => {
                    eprintln!("\n⚠️  Transfer cancelled!");
                    break;
                }
                _ => {
                    // Still in progress
                }
            }
        } else {
            eprintln!("❌ Transfer not found!");
            std::process::exit(1);
        }
    }

    Ok(())
}

