'use client'

import { useState, useEffect } from 'react'
import { Activity, HardDrive, Cpu, MemoryStick, Network, AlertCircle, CheckCircle2, TrendingUp } from 'lucide-react'
import { apiClient, HealthData, NetworkData, TransferStats } from '@/lib/api'
import { POLLING_INTERVALS } from '@/lib/config'
import { formatPercentage } from '@/lib/utils'
import ConnectionStatus from '@/components/shared/ConnectionStatus'

export default function ServerPage() {
  const [health, setHealth] = useState<HealthData | null>(null)
  const [network, setNetwork] = useState<NetworkData | null>(null)
  const [transfers, setTransfers] = useState<TransferStats | null>(null)
  const [available, setAvailable] = useState(false)
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date())

  useEffect(() => {
    const fetchMetrics = async () => {
      const [healthData, networkData, statsData] = await Promise.all([
        apiClient.getHealth(),
        apiClient.getNetwork(),
        apiClient.getStats(),
      ])

      setHealth(healthData)
      setNetwork(networkData)
      setTransfers(statsData?.transfers || null)
      setAvailable(healthData !== null || networkData !== null || statsData !== null)
      setLastUpdate(new Date())
    }

    fetchMetrics()
    const interval = setInterval(fetchMetrics, POLLING_INTERVALS.health)
    return () => clearInterval(interval)
  }, [])

  if (!available) {
    return (
      <div className="min-h-screen bg-black text-white flex items-center justify-center">
        <div className="text-center">
          <AlertCircle className="w-16 h-16 text-primary-red mx-auto mb-4" />
          <h1 className="text-2xl font-bold mb-2">Service Unavailable</h1>
          <p className="text-gray-400">Unable to connect to monitoring service</p>
          <div className="mt-4">
            <ConnectionStatus />
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-black text-white p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <header className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold flex items-center gap-3">
                <Activity className="w-8 h-8 text-primary-green" />
                System Monitor
              </h1>
              <p className="text-gray-400 mt-1">Real-time system performance and health metrics</p>
            </div>
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2 text-sm text-gray-400">
                <CheckCircle2 className="w-4 h-4 text-primary-green" />
                <span>Updated: {lastUpdate.toLocaleTimeString()}</span>
              </div>
              <ConnectionStatus />
            </div>
          </div>
        </header>

        {/* System Health */}
        {health && (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Cpu className="w-5 h-5 text-primary-green" />
                  <span className="text-gray-400">CPU Usage</span>
                </div>
                <span className="text-2xl font-bold text-white">
                  {formatPercentage(health.cpu_usage)}
                </span>
              </div>
              <div className="w-full bg-gray-800 rounded-full h-2">
                <div
                  className="bg-primary-green h-2 rounded-full transition-all"
                  style={{ width: `${health.cpu_usage * 100}%` }}
                />
              </div>
            </div>

            <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <MemoryStick className="w-5 h-5 text-primary-green" />
                  <span className="text-gray-400">Memory Usage</span>
                </div>
                <span className="text-2xl font-bold text-white">
                  {formatPercentage(health.memory_usage)}
                </span>
              </div>
              <div className="w-full bg-gray-800 rounded-full h-2">
                <div
                  className="bg-primary-green h-2 rounded-full transition-all"
                  style={{ width: `${health.memory_usage * 100}%` }}
                />
              </div>
            </div>

            <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <HardDrive className="w-5 h-5 text-primary-green" />
                  <span className="text-gray-400">Buffer Size</span>
                </div>
                <span className="text-2xl font-bold text-white">
                  {health.buffer_size}
                </span>
              </div>
              <div className="text-sm text-gray-400">
                Queue: {health.queue_size}
              </div>
            </div>
          </div>
        )}

        {/* Network Metrics */}
        {network ? (
          <div className="bg-gray-900 border border-gray-800 rounded-lg p-6 mb-8">
            <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
              <Network className="w-6 h-6 text-primary-green" />
              Network Performance
            </h2>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div>
                <div className="text-sm text-gray-400 mb-1">Round Trip Time</div>
                <div className="text-2xl font-bold text-white">
                  {network.rtt_ms.toFixed(1)} ms
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Jitter</div>
                <div className="text-2xl font-bold text-white">
                  {network.jitter_ms.toFixed(1)} ms
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Packet Loss</div>
                <div className="text-2xl font-bold text-white">
                  {formatPercentage(network.loss_rate, 2)}
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Throughput</div>
                <div className="text-2xl font-bold text-white">
                  {network.throughput_mbps.toFixed(1)} Mbps
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="bg-gray-900 border border-yellow-500/30 rounded-lg p-6 mb-8">
            <div className="flex items-center gap-2 text-yellow-400">
              <AlertCircle className="w-5 h-5" />
              <span>Network metrics unavailable</span>
            </div>
          </div>
        )}

        {/* Transfer Statistics */}
        {transfers && (
          <div className="bg-gray-900 border border-gray-800 rounded-lg p-6 mb-8">
            <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
              <TrendingUp className="w-6 h-6 text-primary-green" />
              Transfer Statistics
            </h2>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div>
                <div className="text-sm text-gray-400 mb-1">Total</div>
                <div className="text-2xl font-bold text-white">
                  {transfers.total}
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Active</div>
                <div className="text-2xl font-bold text-primary-green">
                  {transfers.active}
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Completed</div>
                <div className="text-2xl font-bold text-white">
                  {transfers.completed}
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400 mb-1">Failed</div>
                <div className="text-2xl font-bold text-primary-red">
                  {transfers.failed}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Error Rate */}
        {health && (
          <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
            <h2 className="text-xl font-bold mb-4">Error Rate</h2>
            <div className="flex items-center gap-4">
              <div className="flex-1">
                <div className="w-full bg-gray-800 rounded-full h-4">
                  <div
                    className={`h-4 rounded-full transition-all ${
                      health.error_rate > 0.1
                        ? 'bg-primary-red'
                        : health.error_rate > 0.05
                        ? 'bg-yellow-500'
                        : 'bg-primary-green'
                    }`}
                    style={{ width: `${Math.min(health.error_rate * 1000, 100)}%` }}
                  />
                </div>
              </div>
              <div className="text-2xl font-bold text-white">
                {formatPercentage(health.error_rate, 2)}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
