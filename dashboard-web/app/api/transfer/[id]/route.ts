import { NextRequest, NextResponse } from 'next/server'

import { getBackendUrl } from '@/lib/config'

const BACKEND_URL = getBackendUrl()

export async function POST(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const { id } = params
  const url = new URL(request.url)
  const action = url.searchParams.get('action') || 'pause'

  try {
    const endpoint = `${BACKEND_URL}/api/transfer/${id}/${action}`
    const response = await fetch(endpoint, {
      method: 'POST',
    })

    if (!response.ok) {
      return NextResponse.json(
        { error: `Failed to ${action} transfer` },
        { status: response.status }
      )
    }

    return NextResponse.json({
      success: true,
      message: `Transfer ${action}ed successfully`,
    })
  } catch (error) {
    console.error(`Transfer ${action} error:`, error)
    return NextResponse.json(
      { error: 'Backend server not available' },
      { status: 503 }
    )
  }
}
