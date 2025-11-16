'use client'

import { useState, useEffect } from 'react'
import { Download, CheckCircle2, Clock, Shield, FileCheck, Activity } from 'lucide-react'

interface ReceivedFile {
  id: string
  fileName: string
  fileSize: number
  bytesReceived: number
  status: 'receiving' | 'verifying' | 'completed' | 'failed'
  speed: number
  integrityStatus: 'pending' | 'verified' | 'failed'
  blake3Hash?: string
  receivedAt?: Date
}

export default function ReceiverDashboard() {
  const [files, setFiles] = useState<ReceivedFile[]>([
    {
      id: '1',
      fileName: 'dataset.zip',
      fileSize: 104857600,
      bytesReceived: 45678912,
      status: 'receiving',
      speed: 12.5,
      integrityStatus: 'pending',
    },
    {
      id: '2',
      fileName: 'document.pdf',
      fileSize: 5242880,
      bytesReceived: 5242880,
      status: 'completed',
      speed: 0,
      integrityStatus: 'verified',
      blake3Hash: 'a3f5e8b2c9d1f4a7e6b8c2d5f9a1b4c7e8d2f5a9b1c4d7e8f2a5b9c1d4e7f8a2',
      receivedAt: new Date(),
    },
  ])

  const [stats, setStats] = useState({
    totalReceived: 2,
    totalSize: 110100480,
    averageSpeed: 12.5,
    verifiedFiles: 1,
  })

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

  useEffect(() => {
    // Simulate receiving progress
    const interval = setInterval(() => {
      setFiles(prev => prev.map(file => {
        if (file.status === 'receiving' && file.bytesReceived < file.fileSize) {
          const newBytes = Math.min(
            file.bytesReceived + (file.speed * 1024 * 1024 * 0.1),
            file.fileSize
          )
          if (newBytes >= file.fileSize) {
            return {
              ...file,
              bytesReceived: file.fileSize,
              status: 'verifying',
              speed: 0,
            }
          }
          return {
            ...file,
            bytesReceived: newBytes,
          }
        }
        if (file.status === 'verifying') {
          return {
            ...file,
            status: 'completed',
            integrityStatus: 'verified',
            blake3Hash: 'a3f5e8b2c9d1f4a7e6b8c2d5f9a1b4c7e8d2f5a9b1c4d7e8f2a5b9c1d4e7f8a2',
            receivedAt: new Date(),
          }
        }
        return file
      }))
    }, 100)

    return () => clearInterval(interval)
  }, [])

  return (
    <div className="min-h-screen bg-dark-bg p-4 md:p-6 lg:p-8 pt-8">
      {/* Header */}
      <header className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-4xl font-bold neon-text text-primary-green mb-2 flex items-center gap-3">
              <Download className="w-8 h-8" />
              Receiver Dashboard
            </h1>
            <p className="text-gray-400 text-sm">
              Monitor incoming file transfers and verify integrity
            </p>
          </div>
          <div className="flex items-center gap-4">
            <div className="px-4 py-2 rounded-lg glass-card status-active">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 rounded-full bg-primary-green animate-pulse" />
                <span className="text-sm font-medium">Receiving</span>
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <FileCheck className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Total Files</span>
          </div>
          <div className="text-2xl font-bold text-primary-green neon-text">
            {stats.totalReceived}
          </div>
        </div>

        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Activity className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Total Size</span>
          </div>
          <div className="text-2xl font-bold text-primary-green neon-text">
            {formatBytes(stats.totalSize)}
          </div>
        </div>

        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Clock className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Avg Speed</span>
          </div>
          <div className="text-2xl font-bold text-primary-green">
            {stats.averageSpeed.toFixed(1)} MB/s
          </div>
        </div>

        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Shield className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Verified</span>
          </div>
          <div className="text-2xl font-bold text-primary-green neon-text">
            {stats.verifiedFiles}
          </div>
        </div>
      </div>

      {/* Files List */}
      <div className="glass-card p-6">
        <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2 mb-6">
          <FileCheck className="w-6 h-6" />
          Received Files
        </h2>

        <div className="space-y-4">
          {files.map((file) => {
            const progress = (file.bytesReceived / file.fileSize) * 100
            const remaining = file.fileSize - file.bytesReceived
            const eta = file.speed > 0 ? remaining / (file.speed * 1024 * 1024) : 0

            return (
              <div
                key={file.id}
                className="glass-card p-4 border border-dark-border hover:border-primary-green transition-all duration-300"
              >
                {/* File Header */}
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-3">
                    <div className={`p-2 rounded-lg ${
                      file.status === 'completed' ? 'bg-primary-green/20' :
                      file.status === 'verifying' ? 'bg-yellow-500/20' :
                      'bg-primary-green/20'
                    }`}>
                      {file.status === 'completed' ? (
                        <CheckCircle2 className="w-5 h-5 text-primary-green" />
                      ) : file.status === 'verifying' ? (
                        <Shield className="w-5 h-5 text-yellow-400 animate-pulse" />
                      ) : (
                        <Download className="w-5 h-5 text-primary-green" />
                      )}
                    </div>
                    <div>
                      <div className="font-bold text-white">{file.fileName}</div>
                      <div className="text-sm text-gray-400">
                        {formatBytes(file.bytesReceived)} / {formatBytes(file.fileSize)}
                      </div>
                    </div>
                  </div>
                  <div className={`px-3 py-1 rounded-full text-xs font-medium ${
                    file.status === 'completed' ? 'bg-primary-green/20 text-primary-green' :
                    file.status === 'verifying' ? 'bg-yellow-500/20 text-yellow-400' :
                    file.status === 'receiving' ? 'bg-primary-green/20 text-primary-green' :
                    'bg-red-500/20 text-red-400'
                  }`}>
                    {file.status.toUpperCase()}
                  </div>
                </div>

                {/* Progress Bar */}
                {file.status === 'receiving' && (
                  <div className="mb-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm text-gray-400">Progress</span>
                      <span className="text-sm font-bold text-primary-green">
                        {progress.toFixed(1)}%
                      </span>
                    </div>
                    <div className="relative overflow-hidden bg-gray-900 rounded-full h-2">
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
                )}

                {/* File Info */}
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  {file.status === 'receiving' && (
                    <>
                      <div>
                        <div className="text-xs text-gray-400 mb-1">Speed</div>
                        <div className="text-sm font-bold text-primary-green">
                          {file.speed.toFixed(1)} MB/s
                        </div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-400 mb-1">ETA</div>
                        <div className="text-sm font-bold text-primary-green">
                          {formatTime(eta)}
                        </div>
                      </div>
                    </>
                  )}
                  
                  <div>
                    <div className="text-xs text-gray-400 mb-1">Integrity</div>
                    <div className="flex items-center gap-2">
                      {file.integrityStatus === 'verified' ? (
                        <>
                          <CheckCircle2 className="w-4 h-4 text-primary-green" />
                          <span className="text-sm font-medium text-primary-green">Verified</span>
                        </>
                      ) : file.integrityStatus === 'failed' ? (
                        <>
                          <div className="w-4 h-4 border-2 border-red-400 rounded-full" />
                          <span className="text-sm font-medium text-red-400">Failed</span>
                        </>
                      ) : (
                        <>
                          <div className="w-4 h-4 border-2 border-yellow-400 rounded-full animate-pulse" />
                          <span className="text-sm font-medium text-yellow-400">Pending</span>
                        </>
                      )}
                    </div>
                  </div>

                  {file.receivedAt && (
                    <div>
                      <div className="text-xs text-gray-400 mb-1">Received</div>
                      <div className="text-sm text-gray-300">
                        {file.receivedAt.toLocaleTimeString()}
                      </div>
                    </div>
                  )}
                </div>

                {/* BLAKE3 Hash */}
                {file.blake3Hash && (
                  <div className="mt-4 pt-4 border-t border-dark-border">
                    <div className="text-xs text-gray-400 mb-2">BLAKE3 Hash</div>
                    <code className="text-xs text-primary-green font-mono break-all bg-black/30 p-2 rounded">
                      {file.blake3Hash}
                    </code>
                  </div>
                )}
              </div>
            )
          })}
        </div>
      </div>

      {/* Empty State */}
      {files.length === 0 && (
        <div className="glass-card p-12 text-center">
          <Download className="w-16 h-16 text-gray-600 mx-auto mb-4" />
          <h3 className="text-xl font-bold text-gray-400 mb-2">No Files Received</h3>
          <p className="text-gray-500">Waiting for incoming transfers...</p>
        </div>
      )}
    </div>
  )
}

