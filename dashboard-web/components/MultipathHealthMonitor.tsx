'use client'

import { useState, useEffect } from 'react'
import { Wifi, Satellite, Radio, Activity } from 'lucide-react'

interface PathHealth {
  name: string
  type: 'wifi' | '5g' | 'starlink' | 'ethernet'
  rtt: number
  loss: number
  jitter: number
  throughput: number
  status: 'active' | 'backup' | 'down'
  icon: React.ReactNode
}

export default function MultipathHealthMonitor() {
  const [paths, setPaths] = useState<PathHealth[]>([])

  useEffect(() => {
    const fetchPaths = async () => {
      try {
        const response = await fetch('/api/metrics', { cache: 'no-store' })
        if (response.ok) {
          const data = await response.json()
          
          // Check if backend is available
          if (!data.available) {
            setPaths([]) // No paths available
            return
          }
          
          if (data.paths && Array.isArray(data.paths) && data.paths.length > 0) {
            const iconMap = {
              wifi: <Wifi className="w-5 h-5" />,
              '5g': <Radio className="w-5 h-5" />,
              starlink: <Satellite className="w-5 h-5" />,
              ethernet: <Activity className="w-5 h-5" />,
            }
            
            setPaths(data.paths.map((path: any) => ({
              name: path.name || 'Unknown',
              type: path.type || 'wifi',
              rtt: path.rtt || 0,
              loss: path.loss || 0,
              jitter: path.jitter || 0,
              throughput: path.throughput || 0,
              status: path.status || 'down',
              icon: iconMap[path.type as keyof typeof iconMap] || <Activity className="w-5 h-5" />,
            })))
          } else if (data.network) {
            // Try to extract paths from network data
            const networkPaths: PathHealth[] = []
            if (data.network.wifi_available) {
              networkPaths.push({
                name: 'WiFi',
                type: 'wifi',
                rtt: data.network.rtt_ms || 0,
                loss: data.network.loss_rate || 0,
                jitter: data.network.jitter_ms || 0,
                throughput: data.network.throughput_mbps || 0,
                status: 'active',
                icon: <Wifi className="w-5 h-5" />,
              })
            }
            setPaths(networkPaths)
          } else {
            setPaths([]) // No network data available
          }
        }
      } catch (error) {
        console.error('Failed to fetch paths:', error)
        setPaths([]) // Clear paths on error
      }
    }

    fetchPaths()
    const interval = setInterval(fetchPaths, 2000) // Update every 2 seconds
    return () => clearInterval(interval)
  }, [])

  const getHealthColor = (path: PathHealth) => {
    if (path.status === 'down') return 'border-red-500 bg-red-500/10'
    if (path.rtt > 200 || path.loss > 5) return 'border-yellow-500 bg-yellow-500/10'
    if (path.rtt < 50 && path.loss < 1) return 'border-primary-green bg-primary-green/10'
    return 'border-primary-green bg-primary-green/10'
  }

  const getHealthIndicator = (path: PathHealth) => {
    if (path.status === 'down') return 'bg-red-500'
    if (path.rtt > 200 || path.loss > 5) return 'bg-yellow-500'
    if (path.rtt < 50 && path.loss < 1) return 'bg-primary-green'
    return 'bg-primary-green'
  }

  return (
    <div className="glass-card p-6">
      <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2 mb-6">
        <Activity className="w-6 h-6" />
        Multipath Link Health Monitor
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {paths.map((path, index) => (
          <div
            key={index}
            className={`glass-card p-4 border-2 ${getHealthColor(path)} transition-all duration-300`}
          >
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className={`text-primary-green ${path.status === 'active' ? 'animate-pulse' : ''}`}>
                  {path.icon}
                </div>
                <div>
                  <h3 className="font-bold text-white">{path.name}</h3>
                  <div className="flex items-center gap-2 mt-1">
                    <div className={`w-2 h-2 rounded-full ${getHealthIndicator(path)} animate-pulse`} />
                    <span className="text-xs text-gray-400 uppercase">{path.status}</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Metrics Grid */}
            <div className="grid grid-cols-2 gap-3">
              {/* RTT */}
              <div>
                <div className="text-xs text-gray-400 mb-1">RTT</div>
                <div className={`text-lg font-bold ${
                  path.rtt < 50 ? 'text-primary-green' :
                  path.rtt < 200 ? 'text-yellow-400' :
                  'text-red-400'
                }`}>
                  {path.rtt.toFixed(1)} ms
                </div>
              </div>

              {/* Loss */}
              <div>
                <div className="text-xs text-gray-400 mb-1">Loss</div>
                <div className={`text-lg font-bold ${
                  path.loss < 1 ? 'text-primary-green' :
                  path.loss < 5 ? 'text-yellow-400' :
                  'text-red-400'
                }`}>
                  {path.loss.toFixed(2)}%
                </div>
              </div>

              {/* Jitter */}
              <div>
                <div className="text-xs text-gray-400 mb-1">Jitter</div>
                <div className={`text-lg font-bold ${
                  path.jitter < 10 ? 'text-primary-green' :
                  path.jitter < 50 ? 'text-yellow-400' :
                  'text-red-400'
                }`}>
                  {path.jitter.toFixed(1)} ms
                </div>
              </div>

              {/* Throughput */}
              <div>
                <div className="text-xs text-gray-400 mb-1">Throughput</div>
                <div className="text-lg font-bold text-primary-green">
                  {path.throughput.toFixed(1)} Mbps
                </div>
              </div>
            </div>

            {/* Health Bar */}
            <div className="mt-4 h-1 bg-gray-800 rounded-full overflow-hidden">
              <div
                className={`h-full transition-all duration-500 ${
                  path.rtt < 50 && path.loss < 1 ? 'bg-primary-green' :
                  path.rtt < 200 && path.loss < 5 ? 'bg-yellow-400' :
                  'bg-red-500'
                }`}
                style={{
                  width: `${Math.max(0, Math.min(100, 100 - (path.rtt / 10) - (path.loss * 10)))}%`,
                }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

