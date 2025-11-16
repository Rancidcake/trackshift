import { NextRequest, NextResponse } from 'next/server'
import { readFile, writeFile } from 'fs/promises'
import { join } from 'path'

import { getBackendUrl } from '@/lib/config'

const BACKEND_URL = getBackendUrl()
const CONFIG_FILE = join(process.cwd(), 'config.json')

export async function GET() {
  try {
    // Try to fetch from backend first
    try {
      const response = await fetch(`${BACKEND_URL}/api/config`, {
        cache: 'no-store',
      })
      
      if (response.ok) {
        const backendConfig = await response.json()
        return NextResponse.json(backendConfig)
      }
    } catch {
      // Backend not available, use local config
    }

    // Fallback to local config file
    try {
      const config = await readFile(CONFIG_FILE, 'utf-8')
      return NextResponse.json(JSON.parse(config))
    } catch {
      // Return defaults if no config exists
      return NextResponse.json({
        preferredPath: 'auto',
        enableMultipath: true,
        fecAlgorithm: 'reed-solomon',
        fecDataShards: 4,
        fecParityShards: 2,
        compressionEnabled: true,
        compressionLevel: 'lz4',
        defaultPriority: 'normal',
        handoverEnabled: true,
        handoverThreshold: 40,
        chunkSize: 1024 * 1024,
        maxConcurrentTransfers: 3,
      })
    }
  } catch (error) {
    console.error('Config read error:', error)
    return NextResponse.json(
      { error: 'Failed to read config' },
      { status: 500 }
    )
  }
}

export async function POST(request: NextRequest) {
  try {
    const config = await request.json()
    
    // Save to backend if available
    try {
      const response = await fetch(`${BACKEND_URL}/api/config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      })
      
      if (response.ok) {
        // Also save locally as backup
        await writeFile(CONFIG_FILE, JSON.stringify(config, null, 2))
        return NextResponse.json({ success: true, message: 'Configuration saved' })
      }
    } catch {
      // Backend not available, save locally only
      await writeFile(CONFIG_FILE, JSON.stringify(config, null, 2))
      return NextResponse.json({ 
        success: true, 
        message: 'Configuration saved locally (backend not available)' 
      })
    }
    
    return NextResponse.json({ success: true, message: 'Configuration saved' })
  } catch (error) {
    console.error('Config save error:', error)
    return NextResponse.json(
      { error: 'Failed to save config' },
      { status: 500 }
    )
  }
}
