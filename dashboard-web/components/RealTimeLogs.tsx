'use client'

import { useState, useEffect, useRef } from 'react'
import { Terminal, ChevronDown } from 'lucide-react'

interface LogEntry {
  timestamp: string
  level: 'info' | 'warn' | 'error' | 'success'
  message: string
}

export default function RealTimeLogs() {
  const [logs, setLogs] = useState<LogEntry[]>([
    { timestamp: '10:23:45.123', level: 'info', message: 'QUIC connection established' },
    { timestamp: '10:23:45.456', level: 'success', message: 'ECDHE handshake completed' },
    { timestamp: '10:23:45.789', level: 'info', message: 'Kyber-768 key exchange successful' },
    { timestamp: '10:23:46.012', level: 'info', message: 'FEC encoder initialized (Reed-Solomon k=4, r=2)' },
    { timestamp: '10:23:46.234', level: 'warn', message: 'Path handover: WiFi → 5G (RTT spike detected)' },
    { timestamp: '10:23:46.567', level: 'success', message: 'FEC block recovered: 2 missing shards' },
    { timestamp: '10:23:46.890', level: 'info', message: 'Compression: LZ4 applied (ratio: 0.65)' },
  ])

  const [isExpanded, setIsExpanded] = useState(false)
  const logEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // Simulate new log entries
    const interval = setInterval(() => {
      const newLogs: LogEntry[] = [
        { timestamp: new Date().toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 }), level: 'info', message: 'Packet received: sequence 1234' },
        { timestamp: new Date().toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 }), level: 'success', message: 'FEC recovery: block 567 repaired' },
        { timestamp: new Date().toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 }), level: 'info', message: 'Throughput: 125.5 Mbps' },
      ]
      
      setLogs(prev => [...prev.slice(-50), newLogs[Math.floor(Math.random() * newLogs.length)]])
    }, 2000)

    return () => clearInterval(interval)
  }, [])

  // Removed auto-scroll to prevent page jumping

  const getLevelColor = (level: LogEntry['level']) => {
    switch (level) {
      case 'info':
        return 'text-primary-green'
      case 'success':
        return 'text-primary-green'
      case 'warn':
        return 'text-yellow-400'
      case 'error':
        return 'text-red-400'
      default:
        return 'text-gray-400'
    }
  }

  const getLevelPrefix = (level: LogEntry['level']) => {
    switch (level) {
      case 'info':
        return '[INFO]'
      case 'success':
        return '[OK]'
      case 'warn':
        return '[WARN]'
      case 'error':
        return '[ERR]'
      default:
        return '[LOG]'
    }
  }

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold text-primary-green neon-text flex items-center gap-2">
          <Terminal className="w-6 h-6" />
          Real-Time Logs
        </h2>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="p-2 hover:bg-dark-card rounded-lg transition-colors"
        >
          <ChevronDown className={`w-5 h-5 transition-transform ${isExpanded ? 'rotate-180' : ''}`} />
        </button>
      </div>

      <div
        className={`bg-black/50 rounded-lg p-4 font-mono text-sm overflow-y-auto scrollbar-thin transition-all duration-300 ${
          isExpanded ? 'h-96' : 'h-64'
        }`}
      >
        {logs.map((log, index) => (
          <div
            key={index}
            className="mb-1 flex items-start gap-3 hover:bg-white/5 px-2 py-1 rounded"
          >
            <span className="text-gray-500 text-xs min-w-[100px]">{log.timestamp}</span>
            <span className={`font-bold min-w-[60px] ${getLevelColor(log.level)}`}>
              {getLevelPrefix(log.level)}
            </span>
            <span className="text-gray-300 flex-1">{log.message}</span>
          </div>
        ))}
        <div ref={logEndRef} />
      </div>
    </div>
  )
}

