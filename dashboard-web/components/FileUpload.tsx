'use client'

import { useState, useRef, useEffect } from 'react'
import { Upload, X, File, Settings } from 'lucide-react'

interface UploadedFile {
  name: string
  size: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  progress: number
}

export default function FileUpload() {
  const [files, setFiles] = useState<UploadedFile[]>([])
  const [isDragging, setIsDragging] = useState(false)
  const [config, setConfig] = useState<any>(null)
  const [showConfig, setShowConfig] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Load config for this upload
  useEffect(() => {
    const loadConfig = async () => {
      try {
        const response = await fetch('/api/config')
        if (response.ok) {
          const data = await response.json()
          setConfig(data)
        }
      } catch (error) {
        console.error('Failed to load config:', error)
      }
    }
    loadConfig()
  }, [])

  const handleFileSelect = async (selectedFiles: FileList | null) => {
    if (!selectedFiles) return

    const newFiles: UploadedFile[] = Array.from(selectedFiles).map(file => ({
      name: file.name,
      size: file.size,
      status: 'pending' as const,
      progress: 0,
    }))

    setFiles(prev => [...prev, ...newFiles])

    // Upload each file
    for (let i = 0; i < newFiles.length; i++) {
      await uploadFile(selectedFiles[i], newFiles.length - newFiles.length + i)
    }
  }

  const uploadFile = async (file: File, index: number) => {
    const formData = new FormData()
    formData.append('file', file)

    setFiles(prev => {
      const updated = [...prev]
      updated[index] = { ...updated[index], status: 'uploading', progress: 0 }
      return updated
    })

    try {
      const response = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
      })

      if (response.ok) {
        const data = await response.json()
        
        // Start transfer with user's config
        await fetch('/api/transfer', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            filePath: data.fileName,
            remotePath: `/uploads/${data.fileName}`,
            priority: config?.defaultPriority || 'normal',
            path: config?.preferredPath || 'auto',
            fecAlgorithm: config?.fecAlgorithm || 'reed-solomon',
            compression: config?.compressionEnabled ? config.compressionLevel : 'none',
            chunkSize: config?.chunkSize || 1024 * 1024,
          }),
        })

        setFiles(prev => {
          const updated = [...prev]
          updated[index] = { ...updated[index], status: 'completed', progress: 100 }
          return updated
        })
      } else {
        throw new Error('Upload failed')
      }
    } catch (error) {
      setFiles(prev => {
        const updated = [...prev]
        updated[index] = { ...updated[index], status: 'error', progress: 0 }
        return updated
      })
    }
  }

  const removeFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index))
  }

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
  }

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold text-primary-green flex items-center gap-2">
          <Upload className="w-6 h-6" />
          Upload Files
        </h2>
        <button
          onClick={() => setShowConfig(!showConfig)}
          className="px-3 py-2 bg-dark-card border border-dark-border rounded-lg hover:bg-dark-card/80 transition-colors flex items-center gap-2"
        >
          <Settings className="w-4 h-4" />
          <span className="text-sm">Upload Settings</span>
        </button>
      </div>

      {showConfig && config && (
        <div className="mb-4 p-4 bg-dark-card border border-dark-border rounded-lg">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
            <div>
              <span className="text-gray-400">Path:</span>
              <span className="text-white ml-2">{config.preferredPath}</span>
            </div>
            <div>
              <span className="text-gray-400">FEC:</span>
              <span className="text-white ml-2">{config.fecAlgorithm}</span>
            </div>
            <div>
              <span className="text-gray-400">Compression:</span>
              <span className="text-white ml-2">
                {config.compressionEnabled ? config.compressionLevel : 'None'}
              </span>
            </div>
            <div>
              <span className="text-gray-400">Priority:</span>
              <span className="text-white ml-2">{config.defaultPriority}</span>
            </div>
          </div>
        </div>
      )}

      {/* Drop Zone */}
      <div
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          isDragging
            ? 'border-primary-green bg-primary-green/10'
            : 'border-dark-border hover:border-primary-green/50'
        }`}
        onDragOver={(e) => {
          e.preventDefault()
          setIsDragging(true)
        }}
        onDragLeave={() => setIsDragging(false)}
        onDrop={(e) => {
          e.preventDefault()
          setIsDragging(false)
          handleFileSelect(e.dataTransfer.files)
        }}
      >
        <Upload className="w-12 h-12 mx-auto mb-4 text-gray-400" />
        <p className="text-gray-400 mb-2">
          Drag and drop files here, or click to select
        </p>
        <button
          onClick={() => fileInputRef.current?.click()}
          className="px-4 py-2 bg-primary-green text-black rounded-lg font-medium hover:bg-primary-green/80 transition-colors"
        >
          Select Files
        </button>
        <input
          ref={fileInputRef}
          type="file"
          multiple
          className="hidden"
          onChange={(e) => handleFileSelect(e.target.files)}
        />
      </div>

      {/* File List */}
      {files.length > 0 && (
        <div className="mt-6 space-y-2">
          {files.map((file, index) => (
            <div
              key={index}
              className="glass-card p-4 border border-dark-border flex items-center justify-between"
            >
              <div className="flex items-center gap-3 flex-1">
                <File className="w-5 h-5 text-primary-green" />
                <div className="flex-1">
                  <div className="font-medium text-white">{file.name}</div>
                  <div className="text-sm text-gray-400">
                    {formatBytes(file.size)}
                  </div>
                  {file.status === 'uploading' && (
                    <div className="mt-2">
                      <div className="h-1 bg-gray-800 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-primary-green transition-all"
                          style={{ width: `${file.progress}%` }}
                        />
                      </div>
                    </div>
                  )}
                </div>
              </div>
              <div className="flex items-center gap-3">
                <span
                  className={`px-2 py-1 rounded text-xs font-medium ${
                    file.status === 'completed'
                      ? 'bg-primary-green/20 text-primary-green'
                      : file.status === 'error'
                      ? 'bg-primary-red/20 text-primary-red'
                      : file.status === 'uploading'
                      ? 'bg-yellow-500/20 text-yellow-400'
                      : 'bg-gray-500/20 text-gray-400'
                  }`}
                >
                  {file.status.toUpperCase()}
                </span>
                <button
                  onClick={() => removeFile(index)}
                  className="p-1 hover:bg-dark-card rounded transition-colors"
                >
                  <X className="w-4 h-4 text-gray-400" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

