// Shared utility functions - used by both dashboards

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

export function formatTime(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`
  return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`
}

export function formatPercentage(value: number, decimals = 1): string {
  return `${(value * 100).toFixed(decimals)}%`
}

export function getStatusColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'inprogress':
    case 'active':
      return 'text-primary-green'
    case 'completed':
      return 'text-primary-green'
    case 'failed':
    case 'error':
      return 'text-primary-red'
    case 'paused':
      return 'text-yellow-400'
    default:
      return 'text-gray-400'
  }
}

export function getStatusIcon(status: string) {
  const statusLower = status.toLowerCase()
  if (statusLower === 'inprogress' || statusLower === 'active') {
    return '🔄'
  }
  if (statusLower === 'completed') {
    return '✅'
  }
  if (statusLower === 'failed' || statusLower === 'error') {
    return '❌'
  }
  if (statusLower === 'paused') {
    return '⏸️'
  }
  return '⏳'
}
