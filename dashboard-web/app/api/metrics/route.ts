import { NextResponse } from 'next/server'

// Connect to real Rust backend dashboard API
import { getBackendUrl } from '@/lib/config'

const BACKEND_URL = getBackendUrl()

export async function GET() {
  try {
    // Fetch real metrics from Rust backend
    const response = await fetch(`${BACKEND_URL}/api/metrics/current`, {
      cache: 'no-store',
      headers: {
        'Accept': 'application/json',
      },
    })

    if (!response.ok) {
      // Backend not available - return unavailable status
      return NextResponse.json({
        available: false,
        error: 'Backend server not available',
        message: 'Connect to backend server to see real metrics',
      }, { status: 503 })
    }

    const backendData = await response.json()
    
    // Only return real data - no defaults, no fake values
    const metrics: any = {
      available: true,
      timestamp: new Date().toISOString(),
    }
    
    // Only include transfer if it exists and has real data
    if (backendData.transfer && backendData.transfer.file_size > 0) {
      metrics.transfer = {
        fileName: backendData.transfer.file_name,
        fileSize: backendData.transfer.file_size,
        bytesTransferred: backendData.transfer.bytes_transferred,
        speed: backendData.transfer.speed_mbps,
        compressionRatio: backendData.transfer.compression_ratio,
        pqcHandshake: backendData.transfer.pqc_handshake,
        estimatedTime: backendData.transfer.eta_seconds,
        status: backendData.transfer.status,
        latency: backendData.transfer.latency_ms,
        networkLatency: backendData.transfer.network_rtt_ms,
        processingLatency: backendData.transfer.processing_latency_ms,
      }
    }
    
    // Only include paths if they exist and have real data
    if (backendData.paths && Array.isArray(backendData.paths) && backendData.paths.length > 0) {
      metrics.paths = backendData.paths.filter((p: any) => p && p.rtt > 0)
    }
    
    // Only include FEC if it has real data
    if (backendData.fec && backendData.fec.total_blocks > 0) {
      metrics.fec = {
        algorithm: backendData.fec.algorithm,
        dataShards: backendData.fec.data_shards,
        parityShards: backendData.fec.parity_shards,
        totalBlocks: backendData.fec.total_blocks,
        recoveredBlocks: backendData.fec.recovered_blocks,
        failedBlocks: backendData.fec.failed_blocks,
        recoveryRate: backendData.fec.recovery_rate,
      }
    }
    
    // Only include integrity if it has real data
    if (backendData.integrity && backendData.integrity.total_chunks > 0) {
      metrics.integrity = {
        blake3Hash: backendData.integrity.blake3_hash,
        merkleRoot: backendData.integrity.merkle_root,
        status: backendData.integrity.status,
        chunksVerified: backendData.integrity.chunks_verified,
        totalChunks: backendData.integrity.total_chunks,
      }
    }
    
    // Only include network if it has real data
    if (backendData.network && backendData.network.rtt_ms > 0) {
      metrics.network = backendData.network
    }

    return NextResponse.json(metrics)
  } catch (error) {
    console.error('Failed to fetch metrics from backend:', error)
    return NextResponse.json({
      available: false,
      error: 'Failed to connect to backend',
      message: 'Backend server is not running or not accessible',
    }, { status: 503 })
  }
}
