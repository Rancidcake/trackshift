'use client'

import { useState } from 'react'
import { Play, Pause, Square, RotateCcw } from 'lucide-react'

interface TransferControlsProps {
  transferId?: string
  onStart?: () => void
  onPause?: () => void
  onResume?: () => void
  onCancel?: () => void
  onRetry?: () => void
}

export default function TransferControls({
  transferId,
  onStart,
  onPause,
  onResume,
  onCancel,
  onRetry,
}: TransferControlsProps) {
  const [isPaused, setIsPaused] = useState(false)

  const handlePause = async () => {
    if (transferId) {
      try {
        await fetch(`/api/transfer/${transferId}/pause`, { method: 'POST' })
        setIsPaused(true)
        onPause?.()
      } catch (error) {
        console.error('Failed to pause transfer:', error)
      }
    }
  }

  const handleResume = async () => {
    if (transferId) {
      try {
        await fetch(`/api/transfer/${transferId}/resume`, { method: 'POST' })
        setIsPaused(false)
        onResume?.()
      } catch (error) {
        console.error('Failed to resume transfer:', error)
      }
    }
  }

  const handleCancel = async () => {
    if (transferId && confirm('Are you sure you want to cancel this transfer?')) {
      try {
        await fetch(`/api/transfer/${transferId}/cancel`, { method: 'POST' })
        onCancel?.()
      } catch (error) {
        console.error('Failed to cancel transfer:', error)
      }
    }
  }

  const handleRetry = async () => {
    if (transferId) {
      try {
        await fetch(`/api/transfer/${transferId}/retry`, { method: 'POST' })
        onRetry?.()
      } catch (error) {
        console.error('Failed to retry transfer:', error)
      }
    }
  }

  return (
    <div className="glass-card p-4 border border-dark-border">
      <h3 className="text-lg font-bold text-white mb-3">Transfer Controls</h3>
      <div className="flex flex-wrap gap-2">
        {onStart && (
          <button
            onClick={onStart}
            className="px-4 py-2 bg-primary-green text-black rounded-lg font-medium hover:bg-primary-green/80 transition-colors flex items-center gap-2"
          >
            <Play className="w-4 h-4" />
            Start
          </button>
        )}
        
        {isPaused ? (
          <button
            onClick={handleResume}
            className="px-4 py-2 bg-primary-green text-black rounded-lg font-medium hover:bg-primary-green/80 transition-colors flex items-center gap-2"
          >
            <Play className="w-4 h-4" />
            Resume
          </button>
        ) : (
          <button
            onClick={handlePause}
            className="px-4 py-2 bg-yellow-500 text-black rounded-lg font-medium hover:bg-yellow-500/80 transition-colors flex items-center gap-2"
          >
            <Pause className="w-4 h-4" />
            Pause
          </button>
        )}
        
        <button
          onClick={handleCancel}
          className="px-4 py-2 bg-primary-red text-white rounded-lg font-medium hover:bg-primary-red/80 transition-colors flex items-center gap-2"
        >
          <Square className="w-4 h-4" />
          Cancel
        </button>
        
        <button
          onClick={handleRetry}
          className="px-4 py-2 bg-gray-700 text-white rounded-lg font-medium hover:bg-gray-600 transition-colors flex items-center gap-2"
        >
          <RotateCcw className="w-4 h-4" />
          Retry
        </button>
      </div>
    </div>
  )
}

