'use client'

import { useState, useEffect } from 'react'
import { Settings, Wifi, Radio, Satellite, Activity, Shield, Archive, Zap } from 'lucide-react'

interface SystemConfig {
  // Path Selection
  preferredPath: 'wifi' | '5g' | 'starlink' | 'ethernet' | 'auto'
  enableMultipath: boolean
  
  // FEC Settings
  fecAlgorithm: 'reed-solomon' | 'xor'
  fecDataShards: number
  fecParityShards: number
  
  // Compression
  compressionEnabled: boolean
  compressionLevel: 'none' | 'lz4' | 'zstd'
  
  // Priority
  defaultPriority: 'urgent' | 'high' | 'normal' | 'bulk'
  
  // Handover
  handoverEnabled: boolean
  handoverThreshold: number // RTT increase percentage
  
  // Performance
  chunkSize: number
  maxConcurrentTransfers: number
}

export default function SystemControls() {
  const [config, setConfig] = useState<SystemConfig>({
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
    chunkSize: 1024 * 1024, // 1MB
    maxConcurrentTransfers: 3,
  })

  const [isExpanded, setIsExpanded] = useState(false)

  // Save config to backend
  const saveConfig = async () => {
    try {
      const response = await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      })
      if (response.ok) {
        alert('Configuration saved successfully!')
      }
    } catch (error) {
      console.error('Failed to save config:', error)
      alert('Failed to save configuration')
    }
  }

  // Load config from backend
  useEffect(() => {
    const loadConfig = async () => {
      try {
        const response = await fetch('/api/config')
        if (response.ok) {
          const data = await response.json()
          if (data) {
            setConfig(data)
          }
        }
      } catch (error) {
        console.error('Failed to load config:', error)
      }
    }
    loadConfig()
  }, [])

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold text-primary-green flex items-center gap-2">
          <Settings className="w-6 h-6" />
          System Controls
        </h2>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="px-4 py-2 bg-primary-green text-black rounded-lg font-medium hover:bg-primary-green/80 transition-colors"
        >
          {isExpanded ? 'Collapse' : 'Expand'}
        </button>
      </div>

      {isExpanded && (
        <div className="space-y-6">
          {/* Path Selection */}
          <div className="border-b border-dark-border pb-4">
            <h3 className="text-lg font-bold text-white mb-3 flex items-center gap-2">
              <Activity className="w-5 h-5" />
              Network Path Selection
            </h3>
            <div className="space-y-3">
              <div>
                <label className="text-sm text-gray-400 mb-2 block">Preferred Path</label>
                <select
                  value={config.preferredPath}
                  onChange={(e) => setConfig({ ...config, preferredPath: e.target.value as any })}
                  className="w-full bg-dark-card border border-dark-border rounded-lg px-4 py-2 text-white"
                >
                  <option value="auto">Auto (Best Available)</option>
                  <option value="wifi">WiFi</option>
                  <option value="5g">5G</option>
                  <option value="starlink">Starlink</option>
                  <option value="ethernet">Ethernet</option>
                </select>
              </div>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={config.enableMultipath}
                  onChange={(e) => setConfig({ ...config, enableMultipath: e.target.checked })}
                  className="w-4 h-4 text-primary-green"
                />
                <span className="text-sm text-gray-300">Enable Multipath Aggregation</span>
              </label>
            </div>
          </div>

          {/* FEC Settings */}
          <div className="border-b border-dark-border pb-4">
            <h3 className="text-lg font-bold text-white mb-3 flex items-center gap-2">
              <Shield className="w-5 h-5" />
              FEC (Forward Error Correction)
            </h3>
            <div className="space-y-3">
              <div>
                <label className="text-sm text-gray-400 mb-2 block">Algorithm</label>
                <select
                  value={config.fecAlgorithm}
                  onChange={(e) => setConfig({ ...config, fecAlgorithm: e.target.value as any })}
                  className="w-full bg-dark-card border border-dark-border rounded-lg px-4 py-2 text-white"
                >
                  <option value="reed-solomon">Reed-Solomon (Robust)</option>
                  <option value="xor">XOR (Fast)</option>
                </select>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-sm text-gray-400 mb-2 block">Data Shards (k)</label>
                  <input
                    type="number"
                    min="2"
                    max="10"
                    value={config.fecDataShards}
                    onChange={(e) => setConfig({ ...config, fecDataShards: parseInt(e.target.value) })}
                    className="w-full bg-dark-card border border-dark-border rounded-lg px-4 py-2 text-white"
                  />
                </div>
                <div>
                  <label className="text-sm text-gray-400 mb-2 block">Parity Shards (r)</label>
                  <input
                    type="number"
                    min="1"
                    max="5"
                    value={config.fecParityShards}
                    onChange={(e) => setConfig({ ...config, fecParityShards: parseInt(e.target.value) })}
                    className="w-full bg-dark-card border border-dark-border rounded-lg px-4 py-2 text-white"
                  />
                </div>
              </div>
            </div>
          </div>

          {/* Compression */}
          <div className="border-b border-dark-border pb-4">
            <h3 className="text-lg font-bold text-white mb-3 flex items-center gap-2">
              <Archive className="w-5 h-5" />
              Compression
            </h3>
            <div className="space-y-3">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={config.compressionEnabled}
                  onChange={(e) => setConfig({ ...config, compressionEnabled: e.target.checked })}
                  className="w-4 h-4 text-primary-green"
                />
                <span className="text-sm text-gray-300">Enable Compression</span>
              </label>
              {config.compressionEnabled && (
                <div>
                  <label className="text-sm text-gray-400 mb-2 block">Compression Algorithm</label>
                  <select
                    value={config.compressionLevel}
                    onChange={(e) => setConfig({ ...config, compressionLevel: e.target.value as any })}
                    className="w-full bg-dark-card border border-dark-border rounded-lg px-4 py-2 text-white"
                  >
                    <option value="lz4">LZ4 (Fast)</option>
                    <option value="zstd">Zstd (Better Ratio)</option>
                  </select>
                </div>
              )}
            </div>
          </div>

          {/* Priority & Performance */}
          <div className="border-b border-dark-border pb-4">
            <h3 className="text-lg font-bold text-white mb-3 flex items-center gap-2">
              <Zap className="w-5 h-5" />
              Priority & Performance
            </h3>
            <div className="space-y-3">
              <div>
                <label className="text-sm text-gray-400 mb-2 block">Default Priority</label>
                <select
                  value={config.defaultPriority}
                  onChange={(e) => setConfig({ ...config, defaultPriority: e.target.value as any })}
                  className="w-full bg-dark-card border border-dark-border rounded-lg px-4 py-2 text-white"
                >
                  <option value="urgent">Urgent</option>
                  <option value="high">High</option>
                  <option value="normal">Normal</option>
                  <option value="bulk">Bulk</option>
                </select>
              </div>
              <div>
                <label className="text-sm text-gray-400 mb-2 block">
                  Chunk Size: {config.chunkSize / (1024 * 1024)} MB
                </label>
                <input
                  type="range"
                  min="256"
                  max="4096"
                  step="256"
                  value={config.chunkSize / 1024}
                  onChange={(e) => setConfig({ ...config, chunkSize: parseInt(e.target.value) * 1024 })}
                  className="w-full"
                />
              </div>
              <div>
                <label className="text-sm text-gray-400 mb-2 block">Max Concurrent Transfers</label>
                <input
                  type="number"
                  min="1"
                  max="10"
                  value={config.maxConcurrentTransfers}
                  onChange={(e) => setConfig({ ...config, maxConcurrentTransfers: parseInt(e.target.value) })}
                  className="w-full bg-dark-card border border-dark-border rounded-lg px-4 py-2 text-white"
                />
              </div>
            </div>
          </div>

          {/* Handover Settings */}
          <div className="pb-4">
            <h3 className="text-lg font-bold text-white mb-3">Path Handover</h3>
            <div className="space-y-3">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={config.handoverEnabled}
                  onChange={(e) => setConfig({ ...config, handoverEnabled: e.target.checked })}
                  className="w-4 h-4 text-primary-green"
                />
                <span className="text-sm text-gray-300">Enable Automatic Handover</span>
              </label>
              {config.handoverEnabled && (
                <div>
                  <label className="text-sm text-gray-400 mb-2 block">
                    RTT Increase Threshold: {config.handoverThreshold}%
                  </label>
                  <input
                    type="range"
                    min="10"
                    max="100"
                    step="5"
                    value={config.handoverThreshold}
                    onChange={(e) => setConfig({ ...config, handoverThreshold: parseInt(e.target.value) })}
                    className="w-full"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    Handover triggers when RTT increases by this percentage
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-3 pt-4">
            <button
              onClick={saveConfig}
              className="flex-1 px-4 py-2 bg-primary-green text-black rounded-lg font-medium hover:bg-primary-green/80 transition-colors"
            >
              Save Configuration
            </button>
            <button
              onClick={() => {
                // Reset to defaults
                setConfig({
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
              }}
              className="px-4 py-2 bg-gray-700 text-white rounded-lg font-medium hover:bg-gray-600 transition-colors"
            >
              Reset to Defaults
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

