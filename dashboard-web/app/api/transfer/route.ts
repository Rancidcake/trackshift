import { NextRequest, NextResponse } from 'next/server'

import { getBackendUrl } from '@/lib/config'

const BACKEND_URL = getBackendUrl()

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { filePath, remotePath, priority, path, fecAlgorithm, compression, chunkSize } = body

    // Send to Rust backend
    const response = await fetch(`${BACKEND_URL}/api/transfer/start`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        file_path: filePath,
        remote_path: remotePath,
        priority,
        path,
        fec_algorithm: fecAlgorithm,
        compression,
        chunk_size: chunkSize,
      }),
    })

    if (!response.ok) {
      const error = await response.text()
      return NextResponse.json(
        { error: 'Failed to start transfer', details: error },
        { status: response.status }
      )
    }

    const data = await response.json()
    return NextResponse.json({
      success: true,
      transferId: data.transfer_id,
      message: 'Transfer started',
    })
  } catch (error) {
    console.error('Transfer error:', error)
    return NextResponse.json(
      { error: 'Backend server not available', message: 'Cannot start transfer. Please ensure backend server is running.' },
      { status: 503 }
    )
  }
}

export async function GET() {
  try {
    const response = await fetch(`${BACKEND_URL}/api/transfers`, {
      cache: 'no-store',
    })

    if (!response.ok) {
      return NextResponse.json({ transfers: [] }, { status: 200 })
    }

    const data = await response.json()
    return NextResponse.json(data)
  } catch (error) {
    console.error('Failed to fetch transfers:', error)
    return NextResponse.json({ transfers: [] }, { status: 200 })
  }
}
