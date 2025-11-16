'use client'

import { useState, useEffect } from 'react'
import { AlertCircle, CheckCircle2 } from 'lucide-react'
import { apiClient } from '@/lib/api'
import { getBackendUrl } from '@/lib/config'

export default function ConnectionStatus() {
  const [isConnected, setIsConnected] = useState(false)

  useEffect(() => {
    const checkConnection = async () => {
      const connected = await apiClient.checkConnection()
      setIsConnected(connected)
    }

    checkConnection()
    const interval = setInterval(checkConnection, 5000)
    return () => clearInterval(interval)
  }, [])

  return (
    <div className={`px-4 py-2 rounded-lg flex items-center gap-2 ${
      isConnected ? 'bg-primary-green/20 border border-primary-green' : 'bg-primary-red/20 border border-primary-red'
    }`}>
      {isConnected ? (
        <>
          <CheckCircle2 className="w-4 h-4 text-primary-green" />
          <span className="text-sm font-medium text-primary-green">Connected</span>
        </>
      ) : (
        <>
          <AlertCircle className="w-4 h-4 text-primary-red" />
          <span className="text-sm font-medium text-primary-red">Disconnected</span>
        </>
      )}
      <span className="text-xs text-gray-400 ml-2">({getBackendUrl()})</span>
    </div>
  )
}

