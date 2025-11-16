'use client'

import { useState, useEffect } from 'react'
import { FileText, Clock, Zap, Archive, Shield, CheckCircle2 } from 'lucide-react'

interface TransferData {
  fileName: string
  fileSize: number
  bytesTransferred: number
  speed: number
  compressionRatio: number
  pqcHandshake: boolean
  estimatedTime: number
  status: 'active' | 'completed' | 'paused' | 'error'
  latency: number // Total latency in ms
  networkLatency: number // Network RTT in ms
  processingLatency: number // Processing time in ms
}

export default function FileTransferOverview() {
  const [transfer, setTransfer] = useState<TransferData>({
    fileName: 'No active transfer',
    fileSize: 0,
    bytesTransferred: 0,
    speed: 0,
    compressionRatio: 1.0,
    pqcHandshake: false,
    estimatedTime: 0,
    status: 'paused',
    latency: 0,
    networkLatency: 0,
    processingLatency: 0,
  })

  // Fetch real data from API
  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        const response = await fetch('/api/metrics', { cache: 'no-store' })
        if (response.ok) {
          const data = await response.json()
          
          // Check if backend is available
          if (!data.available) {
            // Backend not available - show no active transfer
            setTransfer(prev => ({
              ...prev,
              fileName: 'Backend not available',
              status: 'paused' as const,
            }))
            return
          }
          
          if (data.transfer) {
            setTransfer({
              fileName: data.transfer.fileName || 'No active transfer',
              fileSize: data.transfer.fileSize || 0,
              bytesTransferred: data.transfer.bytesTransferred || 0,
              speed: data.transfer.speed || 0,
              compressionRatio: data.transfer.compressionRatio || 1.0,
              pqcHandshake: data.transfer.pqcHandshake || false,
              estimatedTime: data.transfer.estimatedTime || 0,
              status: data.transfer.status || 'paused',
              latency: data.transfer.latency || 0,
              networkLatency: data.transfer.networkLatency || 0,
              processingLatency: data.transfer.processingLatency || 0,
            })
          } else {
            // No active transfer
            setTransfer({
              fileName: 'No active transfer',
              fileSize: 0,
              bytesTransferred: 0,
              speed: 0,
              compressionRatio: 1.0,
              pqcHandshake: false,
              estimatedTime: 0,
              status: 'paused',
              latency: 0,
              networkLatency: 0,
              processingLatency: 0,
            })
          }
        } else {
          // Backend error
          setTransfer(prev => ({
            ...prev,
            fileName: 'Backend error',
            status: 'error' as const,
          }))
        }
      } catch (error) {
        console.error('Failed to fetch metrics:', error)
        setTransfer(prev => ({
          ...prev,
          fileName: 'Connection failed',
          status: 'error' as const,
        }))
      }
    }

    fetchMetrics()
    const interval = setInterval(fetchMetrics, 1000) // Update every second
    return () => clearInterval(interval)
  }, [])

  const progress = (transfer.bytesTransferred / transfer.fileSize) * 100
  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
  }

  const formatTime = (seconds: number) => {
    if (seconds < 60) return `${Math.round(seconds)}s`
    const mins = Math.floor(seconds / 60)
    const secs = Math.round(seconds % 60)
    return `${mins}m ${secs}s`
  }

  // Real-time updates are now handled by the API fetch above

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2">
          <FileText className="w-6 h-6" />
          File Transfer Overview
        </h2>
        <div className={`px-3 py-1 rounded-full text-xs font-medium ${
          transfer.status === 'active' ? 'bg-primary-green/20 text-primary-green' :
          transfer.status === 'completed' ? 'bg-primary-green/20 text-primary-green' :
          transfer.status === 'paused' ? 'bg-yellow-500/20 text-yellow-400' :
          'bg-red-500/20 text-red-400'
        }`}>
          {transfer.status.toUpperCase()}
        </div>
      </div>

      {/* File Metadata */}
      <div className="mb-6 space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-gray-400">File Name</span>
          <span className="text-white font-mono text-sm">{transfer.fileName}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-gray-400">File Size</span>
          <span className="text-white font-mono">{formatBytes(transfer.fileSize)}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-gray-400">Transferred</span>
          <span className="text-primary-green font-mono">
            {formatBytes(transfer.bytesTransferred)} / {formatBytes(transfer.fileSize)}
          </span>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-gray-400">Progress</span>
          <span className="text-sm font-bold text-primary-green">{progress.toFixed(1)}%</span>
        </div>
        <div className="relative overflow-hidden bg-gray-900 rounded-full h-3">
          <div
            className="h-full bg-primary-green transition-all duration-300 ease-out"
            style={{ width: `${progress}%` }}
            role="progressbar"
            aria-valuenow={progress}
            aria-valuemin={0}
            aria-valuemax={100}
          />
        </div>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {/* Speed */}
        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Zap className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Speed</span>
          </div>
          <div className="text-2xl font-bold text-primary-green neon-text">
            {transfer.speed.toFixed(1)} MB/s
          </div>
        </div>

        {/* Compression */}
        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Archive className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Compression</span>
          </div>
          <div className="text-2xl font-bold text-primary-green neon-text">
            {((1 - transfer.compressionRatio) * 100).toFixed(0)}%
          </div>
          <div className="text-xs text-gray-500 mt-1">
            Savings: {formatBytes(transfer.fileSize * (1 - transfer.compressionRatio))}
          </div>
        </div>

        {/* PQC Handshake */}
        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Shield className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">PQC Handshake</span>
          </div>
          <div className="flex items-center gap-2">
            {transfer.pqcHandshake ? (
              <>
                <CheckCircle2 className="w-5 h-5 text-primary-green" />
                <span className="text-sm font-medium text-primary-green">Active</span>
              </>
            ) : (
              <>
                <div className="w-5 h-5 border-2 border-yellow-400 rounded-full animate-pulse" />
                <span className="text-sm font-medium text-yellow-400">Pending</span>
              </>
            )}
          </div>
          <div className="text-xs text-gray-500 mt-1">Kyber-768 + ECDHE</div>
        </div>

        {/* Estimated Time */}
        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Clock className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">ETA</span>
          </div>
          <div className="text-2xl font-bold text-primary-green">
            {formatTime(transfer.estimatedTime)}
          </div>
        </div>
      </div>

      {/* Latency Breakdown */}
      {transfer.status === 'active' && transfer.latency > 0 && (
        <div className="mt-6 p-4 glass-card border border-dark-border">
          <h3 className="text-sm font-bold text-white mb-3">Latency Breakdown</h3>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <div className="text-xs text-gray-400 mb-1">Total Latency</div>
              <div className="text-lg font-bold text-primary-green">
                {transfer.latency.toFixed(2)} ms
              </div>
            </div>
            <div>
              <div className="text-xs text-gray-400 mb-1">Network RTT</div>
              <div className="text-lg font-bold text-white">
                {transfer.networkLatency.toFixed(2)} ms
              </div>
            </div>
            <div>
              <div className="text-xs text-gray-400 mb-1">Processing</div>
              <div className="text-lg font-bold text-white">
                {transfer.processingLatency.toFixed(2)} ms
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

