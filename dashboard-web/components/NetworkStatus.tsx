'use client'

import { useState, useEffect } from 'react'
import { Activity, AlertCircle } from 'lucide-react'

interface NetworkStatusData {
  available: boolean
  rtt_ms?: number
  throughput_mbps?: number
  loss_rate?: number
  paths_available?: number
}

export default function NetworkStatus() {
  const [status, setStatus] = useState<NetworkStatusData>({ available: false })

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const response = await fetch('/api/metrics', { cache: 'no-store' })
        if (response.ok) {
          const data = await response.json()
          
          if (data.available && data.network) {
            setStatus({
              available: true,
              rtt_ms: data.network.rtt_ms,
              throughput_mbps: data.network.throughput_mbps,
              loss_rate: data.network.loss_rate,
              paths_available: data.paths?.length || 0,
            })
          } else {
            setStatus({ available: false })
          }
        } else {
          setStatus({ available: false })
        }
      } catch {
        setStatus({ available: false })
      }
    }

    fetchStatus()
    const interval = setInterval(fetchStatus, 3000)
    return () => clearInterval(interval)
  }, [])

  if (!status.available) {
    return (
      <div className="glass-card p-4 border border-yellow-500/50">
        <div className="flex items-center gap-2 text-yellow-400">
          <AlertCircle className="w-5 h-5" />
          <span className="font-medium">Network Data Unavailable</span>
        </div>
        <p className="text-sm text-gray-400 mt-2">
          Network interface metrics are not available. File transfers will still work.
        </p>
      </div>
    )
  }

  return (
    <div className="glass-card p-4 border border-dark-border">
      <div className="flex items-center gap-2 mb-3">
        <Activity className="w-5 h-5 text-primary-green" />
        <span className="font-medium text-white">Network Status</span>
      </div>
      <div className="grid grid-cols-3 gap-4 text-sm">
        {status.rtt_ms !== undefined && (
          <div>
            <div className="text-gray-400">RTT</div>
            <div className="text-primary-green font-bold">{status.rtt_ms.toFixed(1)} ms</div>
          </div>
        )}
        {status.throughput_mbps !== undefined && (
          <div>
            <div className="text-gray-400">Throughput</div>
            <div className="text-primary-green font-bold">{status.throughput_mbps.toFixed(1)} Mbps</div>
          </div>
        )}
        {status.loss_rate !== undefined && (
          <div>
            <div className="text-gray-400">Loss</div>
            <div className="text-primary-green font-bold">{(status.loss_rate * 100).toFixed(2)}%</div>
          </div>
        )}
      </div>
      {status.paths_available !== undefined && (
        <div className="mt-3 text-xs text-gray-400">
          {status.paths_available} network path(s) available
        </div>
      )}
    </div>
  )
}

