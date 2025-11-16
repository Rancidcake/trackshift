//! QUIC-FEC File Transfer Server Example
//!
//! This example demonstrates a complete file transfer server with:
//! - TLS 1.3 with ECDHE key exchange (automatic in TLS 1.3)
//! - 3-way handshake for connection establishment
//! - File transfer handling
//! - Session management
//! - Authentication

use anyhow::{Result, Context};
use quic_fec::QuicFecServer;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;

/// Generate self-signed certificate for testing
fn generate_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    // For production, use proper certificates
    // For this example, we'll create a simple self-signed cert
    
    // In a real implementation, you'd use rcgen or similar
    // For now, we'll use a placeholder approach
    
    // Load or generate certificate
    // This is a simplified version - in production use proper certificate generation
    let cert_path = "server.crt";
    let key_path = "server.key";
    
    if std::path::Path::new(cert_path).exists() {
        // Load existing certificate
        let cert_file = File::open(cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
        let cert = certs.first().ok_or_else(|| anyhow::anyhow!("No certificate found"))?;
        
        let key_file = File::open(key_path)?;
        let mut key_reader = BufReader::new(key_file);
        let keys = pkcs8_private_keys(&mut key_reader).collect::<Result<Vec<_>, _>>()?;
        let key = keys.first().ok_or_else(|| anyhow::anyhow!("No private key found"))?;
        
        Ok((
            CertificateDer::from(cert.to_vec()),
            PrivateKeyDer::from(key.key_der().to_vec()),
        ))
    } else {
        // Generate self-signed certificate (simplified - use rcgen in production)
        eprintln!("⚠️  Certificate files not found. Generating self-signed certificate...");
        eprintln!("   For production, use proper certificates!");
        
        // This is a placeholder - in production, use rcgen crate
        anyhow::bail!("Please generate certificate files: {} and {}", cert_path, key_path);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 QUIC-FEC File Transfer Server");
    println!("================================\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let addr: SocketAddr = args
        .get(1)
        .map(|s| s.parse())
        .unwrap_or_else(|| "127.0.0.1:8080".parse())
        .context("Invalid address")?;

    let storage_path = PathBuf::from(
        args.get(2)
            .map(|s| s.as_str())
            .unwrap_or("server_storage"),
    );

    println!("📡 Server address: {}", addr);
    println!("📁 Storage path: {}", storage_path.display());
    println!();

    // Generate or load certificate
    let (cert, key) = generate_cert()
        .context("Failed to load/generate certificate")?;

    println!("🔐 Certificate loaded");
    println!();

    // Create server
    let server = QuicFecServer::new(addr, cert, key, storage_path)
        .context("Failed to create server")?;

    println!("✅ Server initialized");
    println!("📡 Listening for connections...\n");

    // Run server
    if let Err(e) = server.run().await {
        eprintln!("❌ Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

