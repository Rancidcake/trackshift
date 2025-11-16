// Centralized API client - all API calls go through here

import { getBackendUrl, API_ENDPOINTS } from './config'

export interface HealthData {
  cpu_usage: number
  memory_usage: number
  active_transfers: number
  queue_size: number
  buffer_size: number
  error_rate: number
  timestamp: number
}

export interface NetworkData {
  rtt_ms: number
  jitter_ms: number
  loss_rate: number
  throughput_mbps: number
  wifi_signal?: number
  starlink_latency?: number
  quality_score?: number
  is_patchy?: boolean
  timestamp?: number
}

export interface Transfer {
  id: string
  status: string
  progress: number
  bytes_transferred: number
  total_bytes: number
  speed_mbps: number
  eta_seconds?: number
  priority: string
  route?: string
  integrity_method?: string
  retry_count?: number
  error_message?: string
}

export interface TransferStats {
  total: number
  active: number
  completed: number
  failed: number
}

export interface MetricsData {
  available: boolean
  health?: HealthData
  network?: NetworkData
  transfers?: TransferStats
  error?: string
}

class ApiClient {
  private baseUrl: string

  constructor() {
    this.baseUrl = getBackendUrl()
  }

  async fetchWithTimeout(url: string, options: RequestInit = {}, timeout = 3000): Promise<Response> {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), timeout)
    
    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
        cache: 'no-store',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
          ...options.headers,
        },
      })
      clearTimeout(timeoutId)
      return response
    } catch (error) {
      clearTimeout(timeoutId)
      throw error
    }
  }

  async getHealth(): Promise<HealthData | null> {
    try {
      const response = await this.fetchWithTimeout(API_ENDPOINTS.health())
      if (response.ok) {
        return await response.json()
      }
      return null
    } catch {
      return null
    }
  }

  async getNetwork(): Promise<NetworkData | null> {
    try {
      const response = await this.fetchWithTimeout(API_ENDPOINTS.network())
      if (response.ok) {
        const data = await response.json()
        if (data.status === 'no_data') {
          return null
        }
        return data
      }
      return null
    } catch {
      return null
    }
  }

  async getTransfers(): Promise<Transfer[]> {
    try {
      const response = await this.fetchWithTimeout(API_ENDPOINTS.transfers())
      if (response.ok) {
        const data = await response.json()
        return data.transfers || []
      }
      return []
    } catch {
      return []
    }
  }

  async getStats(): Promise<{ transfers: TransferStats } | null> {
    try {
      const response = await this.fetchWithTimeout(API_ENDPOINTS.stats())
      if (response.ok) {
        const data = await response.json()
        return data
      }
      return null
    } catch {
      return null
    }
  }

  async getMetrics(): Promise<MetricsData> {
    try {
      const response = await this.fetchWithTimeout(API_ENDPOINTS.metrics())
      if (response.ok) {
        const data = await response.json()
        return {
          available: true,
          health: data.health,
          network: data.network,
          transfers: data.transfers,
        }
      }
      return { available: false, error: 'Backend not available' }
    } catch (error) {
      return { available: false, error: 'Connection failed' }
    }
  }

  async checkConnection(): Promise<boolean> {
    try {
      const response = await this.fetchWithTimeout(API_ENDPOINTS.health(), {}, 2000)
      return response.ok
    } catch {
      return false
    }
  }
}

// Singleton instance
export const apiClient = new ApiClient()

