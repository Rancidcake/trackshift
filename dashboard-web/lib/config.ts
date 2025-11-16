// Centralized configuration - no hardcoded values

export const getBackendUrl = (): string => {
  if (typeof window !== 'undefined') {
    // Client-side: use public env var
    return process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
  }
  // Server-side: use server env var
  return process.env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
}

export const API_ENDPOINTS = {
  health: () => `${getBackendUrl()}/api/health`,
  network: () => `${getBackendUrl()}/api/network`,
  transfers: () => `${getBackendUrl()}/api/transfers`,
  stats: () => `${getBackendUrl()}/api/stats`,
  config: () => `${getBackendUrl()}/api/config`,
  metrics: () => `${getBackendUrl()}/api/metrics/current`,
} as const

export const POLLING_INTERVALS = {
  transfers: 1000, // 1 second
  health: 2000, // 2 seconds
  network: 2000, // 2 seconds
  stats: 3000, // 3 seconds
} as const

