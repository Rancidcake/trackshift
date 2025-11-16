'use client'

import { Transfer } from '@/lib/api'
import { formatBytes, formatTime, getStatusColor, getStatusIcon } from '@/lib/utils'
import { Clock, CheckCircle2, XCircle, AlertCircle } from 'lucide-react'

interface TransferListProps {
  transfers: Transfer[]
  showActiveOnly?: boolean
}

export default function TransferList({ transfers, showActiveOnly = false }: TransferListProps) {
  const displayTransfers = showActiveOnly
    ? transfers.filter(t => t.status.toLowerCase() === 'inprogress')
    : transfers

  const getStatusIconComponent = (status: string) => {
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

  if (displayTransfers.length === 0) {
    return (
      <div className="bg-gray-900 border border-gray-800 rounded-lg p-8 text-center">
        <p className="text-gray-400">No transfers {showActiveOnly ? 'active' : 'available'}</p>
      </div>
    )
  }

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-lg overflow-hidden">
      <table className="w-full">
        <thead className="bg-gray-800">
          <tr>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase">ID</th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase">Status</th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase">Progress</th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase">Speed</th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase">Priority</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-800">
          {displayTransfers.map((transfer) => (
            <tr key={transfer.id}>
              <td className="px-6 py-4 text-sm text-white">{transfer.id}</td>
              <td className="px-6 py-4">
                <div className="flex items-center gap-2">
                  {getStatusIconComponent(transfer.status)}
                  <span className={`text-sm ${getStatusColor(transfer.status)}`}>
                    {transfer.status}
                  </span>
                </div>
              </td>
              <td className="px-6 py-4">
                <div className="flex items-center gap-2">
                  <div className="w-24 bg-gray-800 rounded-full h-2">
                    <div
                      className="bg-primary-green h-2 rounded-full transition-all"
                      style={{ width: `${transfer.progress * 100}%` }}
                    />
                  </div>
                  <span className="text-sm text-gray-400">
                    {(transfer.progress * 100).toFixed(1)}%
                  </span>
                </div>
              </td>
              <td className="px-6 py-4 text-sm text-white">
                {transfer.speed_mbps.toFixed(2)} MB/s
              </td>
              <td className="px-6 py-4 text-sm text-gray-400">{transfer.priority}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

