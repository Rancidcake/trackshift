'use client'

import { useState, useEffect } from 'react'
import { ShieldCheck, CheckCircle2, XCircle, Hash } from 'lucide-react'

interface IntegrityData {
  blake3Hash: string
  merkleRoot: string
  status: 'verified' | 'failed' | 'pending'
  chunksVerified: number
  totalChunks: number
}

export default function IntegrityVerification() {
  const [integrity, setIntegrity] = useState<IntegrityData>({
    blake3Hash: 'a3f5e8b2c9d1f4a7e6b8c2d5f9a1b4c7e8d2f5a9b1c4d7e8f2a5b9c1d4e7f8a2',
    merkleRoot: '7b9c2d5f8a1b4e7c0d3f6a9b2c5e8d1f4a7b0c3d6e9f2a5b8c1d4e7f0a3b6',
    status: 'verified',
    chunksVerified: 1247,
    totalChunks: 1250,
  })

  const verificationProgress = (integrity.chunksVerified / integrity.totalChunks) * 100

  return (
    <div className="glass-card p-6">
      <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2 mb-6">
        <ShieldCheck className="w-6 h-6" />
        Integrity Verification
      </h2>

      {/* Status */}
      <div className="mb-6">
        <div className={`flex items-center gap-3 p-4 rounded-lg border-2 ${
          integrity.status === 'verified'
            ? 'border-primary-green bg-primary-green/10'
            : integrity.status === 'failed'
            ? 'border-red-500 bg-red-500/10'
            : 'border-yellow-500 bg-yellow-500/10'
        }`}>
          {integrity.status === 'verified' ? (
            <>
              <CheckCircle2 className="w-8 h-8 text-primary-green" />
              <div>
                <div className="font-bold text-primary-green">Verified</div>
                <div className="text-sm text-gray-400">All integrity checks passed</div>
              </div>
            </>
          ) : integrity.status === 'failed' ? (
            <>
              <XCircle className="w-8 h-8 text-red-400" />
              <div>
                <div className="font-bold text-red-400">Verification Failed</div>
                <div className="text-sm text-gray-400">Hash mismatch detected</div>
              </div>
            </>
          ) : (
            <>
              <div className="w-8 h-8 border-2 border-yellow-400 rounded-full animate-spin" />
              <div>
                <div className="font-bold text-yellow-400">Verifying...</div>
                <div className="text-sm text-gray-400">Checking integrity</div>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Progress */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-gray-400">Chunks Verified</span>
          <span className="text-sm font-bold text-primary-green">
            {integrity.chunksVerified} / {integrity.totalChunks}
          </span>
        </div>
        <div className="relative h-2 bg-gray-900 rounded-full overflow-hidden">
          <div
            className={`h-full transition-all duration-500 ${
              integrity.status === 'verified' ? 'bg-primary-green' : 'bg-yellow-400'
            }`}
            style={{ width: `${verificationProgress}%` }}
          />
        </div>
      </div>

      {/* BLAKE3 Hash */}
      <div className="mb-4">
        <div className="flex items-center gap-2 mb-2">
          <Hash className="w-4 h-4 text-primary-green" />
          <span className="text-sm text-gray-400">BLAKE3 Hash</span>
        </div>
        <div className="glass-card p-3 border border-dark-border">
          <code className="text-xs text-primary-green font-mono break-all">
            {integrity.blake3Hash}
          </code>
        </div>
      </div>

      {/* Merkle Tree Root */}
      <div>
        <div className="flex items-center gap-2 mb-2">
          <Hash className="w-4 h-4 text-primary-green" />
          <span className="text-sm text-gray-400">Merkle Tree Root</span>
        </div>
        <div className="glass-card p-3 border border-dark-border">
          <code className="text-xs text-primary-green font-mono break-all">
            {integrity.merkleRoot}
          </code>
        </div>
      </div>
    </div>
  )
}

