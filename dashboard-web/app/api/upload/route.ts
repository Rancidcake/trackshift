import { NextRequest, NextResponse } from 'next/server'
import { writeFile } from 'fs/promises'
import { join } from 'path'

import { getBackendUrl } from '@/lib/config'

const BACKEND_URL = getBackendUrl()

export async function POST(request: NextRequest) {
  try {
    const formData = await request.formData()
    const file = formData.get('file') as File
    
    if (!file) {
      return NextResponse.json({ error: 'No file provided' }, { status: 400 })
    }

    const bytes = await file.arrayBuffer()
    const buffer = Buffer.from(bytes)

    // Save file temporarily
    const uploadsDir = join(process.cwd(), 'uploads')
    await writeFile(join(uploadsDir, file.name), buffer)

    // Get config from request
    const priority = formData.get('priority') as string || 'normal'
    const path = formData.get('path') as string || 'auto'
    const fecAlgorithm = formData.get('fecAlgorithm') as string || 'reed-solomon'
    const compression = formData.get('compression') as string || 'none'
    const chunkSize = parseInt(formData.get('chunkSize') as string || '1048576')

    // Send to Rust backend for transfer
    try {
      const transferResponse = await fetch(`${BACKEND_URL}/api/transfer/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          file_path: join(uploadsDir, file.name),
          remote_path: `/uploads/${file.name}`,
          priority,
          path,
          fec_algorithm: fecAlgorithm,
          compression,
          chunk_size: chunkSize,
        }),
      })

      if (!transferResponse.ok) {
        throw new Error('Backend transfer failed')
      }

      const transferData = await transferResponse.json()

      return NextResponse.json({
        success: true,
        fileName: file.name,
        fileSize: file.size,
        transferId: transferData.transfer_id,
        message: 'File uploaded and transfer started',
      })
    } catch (backendError) {
      // Backend not available - return error
      return NextResponse.json({
        error: 'Backend server not available',
        message: 'Cannot start transfer. Please ensure backend server is running.',
      }, { status: 503 })
    }
  } catch (error) {
    console.error('Upload error:', error)
    return NextResponse.json(
      { error: 'Failed to upload file' },
      { status: 500 }
    )
  }
}
