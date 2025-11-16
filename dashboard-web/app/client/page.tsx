'use client'

import { useState, useEffect } from 'react'
import { Upload, FileText, Zap, AlertCircle, Activity } from 'lucide-react'
import { apiClient, Transfer } from '@/lib/api'
import { POLLING_INTERVALS } from '@/lib/config'
import { formatBytes } from '@/lib/utils'
import TransferCard from '@/components/shared/TransferCard'
import TransferList from '@/components/shared/TransferList'
import ConnectionStatus from '@/components/shared/ConnectionStatus'

export default function ClientPage() {
  const [transfers, setTransfers] = useState<Transfer[]>([])
  const [available, setAvailable] = useState(false)
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [uploading, setUploading] = useState(false)

  useEffect(() => {
    const fetchTransfers = async () => {
      const transferList = await apiClient.getTransfers()
      setTransfers(transferList)
      setAvailable(transferList.length > 0 || await apiClient.checkConnection())
    }

    fetchTransfers()
    const interval = setInterval(fetchTransfers, POLLING_INTERVALS.transfers)
    return () => clearInterval(interval)
  }, [])

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files && event.target.files[0]) {
      setSelectedFile(event.target.files[0])
    }
  }

  const handleUpload = async () => {
    if (!selectedFile) return

    setUploading(true)
    try {
      const formData = new FormData()
      formData.append('file', selectedFile)

      const response = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
      })

      if (response.ok) {
        setSelectedFile(null)
        // Refresh transfers
        const transferList = await apiClient.getTransfers()
        setTransfers(transferList)
      }
    } catch (error) {
      console.error('Upload failed:', error)
    } finally {
      setUploading(false)
    }
  }

  if (!available) {
    return (
      <div className="min-h-screen bg-black text-white flex items-center justify-center">
        <div className="text-center">
          <AlertCircle className="w-16 h-16 text-primary-red mx-auto mb-4" />
          <h1 className="text-2xl font-bold mb-2">Connection Error</h1>
          <p className="text-gray-400">Unable to connect to transfer service</p>
          <ConnectionStatus />
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-black text-white p-6">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <header className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold flex items-center gap-3">
                <Upload className="w-8 h-8 text-primary-green" />
                File Transfer
              </h1>
              <p className="text-gray-400 mt-1">Upload and manage your file transfers</p>
            </div>
            <ConnectionStatus />
          </div>
        </header>

        {/* File Upload */}
        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6 mb-8">
          <h2 className="text-xl font-bold mb-4">Upload File</h2>
          <div className="flex gap-4">
            <input
              type="file"
              onChange={handleFileSelect}
              className="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-primary-green/20 file:text-primary-green hover:file:bg-primary-green/30"
            />
            <button
              onClick={handleUpload}
              disabled={!selectedFile || uploading}
              className={`px-6 py-2 rounded-lg font-semibold transition-colors ${
                !selectedFile || uploading
                  ? 'bg-gray-700 text-gray-400 cursor-not-allowed'
                  : 'bg-primary-green text-black hover:bg-primary-green/80'
              }`}
            >
              {uploading ? 'Uploading...' : 'Upload'}
            </button>
          </div>
          {selectedFile && (
            <div className="mt-4 text-sm text-gray-400">
              Selected: {selectedFile.name} ({formatBytes(selectedFile.size)})
            </div>
          )}
        </div>

        {/* Active Transfers */}
        <div className="mb-8">
          <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
            <Zap className="w-6 h-6 text-primary-green" />
            Active Transfers ({transfers.filter(t => t.status.toLowerCase() === 'inprogress').length})
          </h2>
          {transfers.filter(t => t.status.toLowerCase() === 'inprogress').length === 0 ? (
            <div className="bg-gray-900 border border-gray-800 rounded-lg p-8 text-center">
              <FileText className="w-12 h-12 text-gray-600 mx-auto mb-4" />
              <p className="text-gray-400">No active transfers</p>
            </div>
          ) : (
            <div className="space-y-4">
              {transfers
                .filter(t => t.status.toLowerCase() === 'inprogress')
                .map((transfer) => (
                  <TransferCard key={transfer.id} transfer={transfer} />
                ))}
            </div>
          )}
        </div>

        {/* All Transfers */}
        <div>
          <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
            <Activity className="w-6 h-6 text-primary-green" />
            Transfer History ({transfers.length})
          </h2>
          <TransferList transfers={transfers} />
        </div>
      </div>
    </div>
  )
}
