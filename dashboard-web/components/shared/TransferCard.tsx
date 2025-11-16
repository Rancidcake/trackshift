'use client'

import { Transfer } from '@/lib/api'
import { formatBytes, formatTime } from '@/lib/utils'
import { Clock, CheckCircle2, XCircle, AlertCircle } from 'lucide-react'

interface TransferCardProps {
  transfer: Transfer
}

export default function TransferCard({ transfer }: TransferCardProps) {
  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case 'inprogress':
        return <Clock className="w-5 h-5 text-primary-green animate-spin" />
      case 'completed':
        return <CheckCircle2 className="w-5 h-5 text-primary-green" />
      case 'failed':
        return <XCircle className="w-5 h-5 text-primary-red" />
      default:
        return <AlertCircle className="w-5 h-5 text-gray-400" />
    }
  }

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          {getStatusIcon(transfer.status)}
          <div>
            <div className="font-semibold">Transfer {transfer.id}</div>
            <div className="text-sm text-gray-400">
              {formatBytes(transfer.bytes_transferred)} / {formatBytes(transfer.total_bytes)}
            </div>
          </div>
        </div>
        <div className="text-right">
          <div className="text-lg font-bold text-primary-green">
            {transfer.speed_mbps.toFixed(2)} MB/s
          </div>
          {transfer.eta_seconds && (
            <div className="text-sm text-gray-400">
              ETA: {formatTime(transfer.eta_seconds)}
            </div>
          )}
        </div>
      </div>
      <div className="w-full bg-gray-800 rounded-full h-3">
        <div
          className="bg-primary-green h-3 rounded-full transition-all"
          style={{ width: `${transfer.progress * 100}%` }}
        />
      </div>
      <div className="mt-2 text-xs text-gray-400">
        Priority: {transfer.priority}
      </div>
    </div>
  )
}

