'use client'

import { useState, useEffect } from 'react'
import { BarChart3, TrendingUp, Activity, Zap } from 'lucide-react'
import { LineChart, Line, AreaChart, Area, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'

export default function Analytics() {
  const [throughputData, setThroughputData] = useState<Array<{ time: string; value: number }>>([])
  const [recoveryData, setRecoveryData] = useState<Array<{ time: string; rate: number }>>([])
  const [failoverData, setFailoverData] = useState<Array<{ time: string; count: number }>>([])
  const [compressionData, setCompressionData] = useState<Array<{ time: string; ratio: number }>>([])

  useEffect(() => {
    // Generate sample data
    const generateData = () => {
      const now = new Date()
      const throughput: Array<{ time: string; value: number }> = []
      const recovery: Array<{ time: string; rate: number }> = []
      const failover: Array<{ time: string; count: number }> = []
      const compression: Array<{ time: string; ratio: number }> = []

      for (let i = 11; i >= 0; i--) {
        const time = new Date(now.getTime() - i * 60000)
        throughput.push({
          time: time.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
          value: 100 + Math.random() * 50,
        })
        recovery.push({
          time: time.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
          rate: 95 + Math.random() * 5,
        })
        failover.push({
          time: time.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
          count: Math.floor(Math.random() * 5),
        })
        compression.push({
          time: time.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
          ratio: 0.6 + Math.random() * 0.3,
        })
      }

      setThroughputData(throughput)
      setRecoveryData(recovery)
      setFailoverData(failover)
      setCompressionData(compression)
    }

    generateData()
    const interval = setInterval(() => {
      // Update last data point
      setThroughputData(prev => {
        const newData = [...prev]
        newData[newData.length - 1] = {
          ...newData[newData.length - 1],
          value: 100 + Math.random() * 50,
        }
        return newData
      })
    }, 5000)
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="glass-card p-6">
      <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2 mb-6">
        <BarChart3 className="w-6 h-6" />
        Analytics
      </h2>

      <div className="space-y-6">
        {/* Throughput Chart */}
        <div>
          <div className="flex items-center gap-2 mb-3">
            <TrendingUp className="w-4 h-4 text-primary-green" />
            <span className="text-sm font-medium">Throughput (Mbps)</span>
          </div>
          <div className="h-32">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={throughputData}>
                <defs>
                  <linearGradient id="throughputGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#00ff00" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#00ff00" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <Area
                  type="monotone"
                  dataKey="value"
                  stroke="#00ff00"
                  fill="url(#throughputGradient)"
                  strokeWidth={2}
                />
                <XAxis dataKey="time" tick={{ fill: '#9ca3af', fontSize: 10 }} />
                <YAxis tick={{ fill: '#9ca3af', fontSize: 10 }} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Recovery Rate Chart */}
        <div>
          <div className="flex items-center gap-2 mb-3">
            <Activity className="w-4 h-4 text-primary-green" />
            <span className="text-sm font-medium">FEC Recovery Rate (%)</span>
          </div>
          <div className="h-32">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={recoveryData}>
                <Line
                  type="monotone"
                  dataKey="rate"
                  stroke="#00ff00"
                  strokeWidth={2}
                  dot={false}
                />
                <XAxis dataKey="time" tick={{ fill: '#9ca3af', fontSize: 10 }} />
                <YAxis domain={[90, 100]} tick={{ fill: '#9ca3af', fontSize: 10 }} />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Failover Count */}
        <div>
          <div className="flex items-center gap-2 mb-3">
            <Zap className="w-4 h-4 text-yellow-400" />
            <span className="text-sm font-medium">Failover Events</span>
          </div>
          <div className="h-32">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={failoverData}>
                <Bar dataKey="count" fill="#ffd700" radius={[4, 4, 0, 0]} />
                <XAxis dataKey="time" tick={{ fill: '#9ca3af', fontSize: 10 }} />
                <YAxis tick={{ fill: '#9ca3af', fontSize: 10 }} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Compression Impact */}
        <div>
          <div className="flex items-center gap-2 mb-3">
            <Activity className="w-4 h-4 text-primary-green" />
            <span className="text-sm font-medium">Compression Ratio</span>
          </div>
          <div className="h-32">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={compressionData}>
                <defs>
                  <linearGradient id="compressionGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#00ff00" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#00ff00" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <Area
                  type="monotone"
                  dataKey="ratio"
                  stroke="#00ff00"
                  fill="url(#compressionGradient)"
                  strokeWidth={2}
                />
                <XAxis dataKey="time" tick={{ fill: '#9ca3af', fontSize: 10 }} />
                <YAxis domain={[0.5, 1.0]} tick={{ fill: '#9ca3af', fontSize: 10 }} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  )
}

