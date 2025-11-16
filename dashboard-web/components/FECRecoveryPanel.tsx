'use client'

import { useState, useEffect } from 'react'
import { Shield, CheckCircle2, XCircle, Activity } from 'lucide-react'

interface FECStats {
  algorithm: 'Reed-Solomon' | 'XOR'
  dataShards: number
  parityShards: number
  totalBlocks: number
  recoveredBlocks: number
  failedBlocks: number
  recoveryRate: number
}

export default function FECRecoveryPanel() {
  const [fecStats, setFecStats] = useState<FECStats>({
    algorithm: 'Reed-Solomon',
    dataShards: 4,
    parityShards: 2,
    totalBlocks: 1250,
    recoveredBlocks: 47,
    failedBlocks: 3,
    recoveryRate: 97.6,
  })

  const [heatmap, setHeatmap] = useState<boolean[]>([])

  useEffect(() => {
    // Generate heatmap data (20x10 grid)
    const generateHeatmap = () => {
      const data: boolean[] = []
      for (let i = 0; i < 200; i++) {
        data.push(Math.random() > 0.15) // 85% success rate
      }
      setHeatmap(data)
    }
    generateHeatmap()
    const interval = setInterval(generateHeatmap, 3000)
    return () => clearInterval(interval)
  }, [])

  const recoveryRate = fecStats.totalBlocks > 0
    ? (fecStats.recoveredBlocks / fecStats.totalBlocks) * 100
    : 0

  return (
    <div className="glass-card p-6">
      <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2 mb-6">
        <Shield className="w-6 h-6" />
        FEC Recovery Panel
      </h2>

      {/* FEC Scheme Info */}
      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="glass-card p-4 border border-dark-border">
          <div className="text-xs text-gray-400 mb-2">Algorithm</div>
          <div className="text-xl font-bold text-primary-green neon-text">
            {fecStats.algorithm}
          </div>
        </div>
        <div className="glass-card p-4 border border-dark-border">
          <div className="text-xs text-gray-400 mb-2">Block Configuration</div>
          <div className="text-xl font-bold text-primary-green">
            k={fecStats.dataShards} + r={fecStats.parityShards}
          </div>
        </div>
      </div>

      {/* Recovery Stats */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <CheckCircle2 className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Recovered</span>
          </div>
          <div className="text-2xl font-bold text-primary-green neon-text">
            {fecStats.recoveredBlocks}
          </div>
          <div className="text-xs text-gray-500 mt-1">
            {recoveryRate.toFixed(1)}% success
          </div>
        </div>

        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <XCircle className="w-4 h-4 text-red-400" />
            <span className="text-xs text-gray-400">Failed</span>
          </div>
          <div className="text-2xl font-bold text-red-400">
            {fecStats.failedBlocks}
          </div>
          <div className="text-xs text-gray-500 mt-1">
            {((fecStats.failedBlocks / fecStats.totalBlocks) * 100).toFixed(1)}% failure
          </div>
        </div>

        <div className="glass-card p-4 border border-dark-border">
          <div className="flex items-center gap-2 mb-2">
            <Activity className="w-4 h-4 text-primary-green" />
            <span className="text-xs text-gray-400">Total Blocks</span>
          </div>
          <div className="text-2xl font-bold text-primary-green">
            {fecStats.totalBlocks}
          </div>
          <div className="text-xs text-gray-500 mt-1">
            Processed
          </div>
        </div>
      </div>

      {/* Recovery Rate Progress */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-gray-400">Recovery Rate</span>
          <span className="text-sm font-bold text-primary-green">{recoveryRate.toFixed(1)}%</span>
        </div>
        <div className="relative h-3 bg-gray-900 rounded-full overflow-hidden">
          <div
            className="h-full bg-primary-green transition-all duration-500"
            style={{ width: `${recoveryRate}%` }}
          />
        </div>
      </div>

      {/* Heatmap */}
      <div>
        <div className="text-sm text-gray-400 mb-3">Repaired Chunks Heatmap</div>
        <div className="grid grid-cols-10 gap-1">
          {heatmap.map((repaired, index) => (
            <div
              key={index}
              className={`aspect-square rounded transition-all duration-300 ${
                repaired
                  ? 'bg-primary-green/80 hover:bg-primary-green'
                  : 'bg-red-500/50 hover:bg-red-500'
              }`}
              title={repaired ? 'Repaired' : 'Failed'}
            />
          ))}
        </div>
        <div className="flex items-center gap-4 mt-3 text-xs text-gray-500">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-primary-green/80 rounded" />
            <span>Repaired</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-red-500/50 rounded" />
            <span>Failed</span>
          </div>
        </div>
      </div>
    </div>
  )
}

