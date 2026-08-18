import axios from 'axios'
import { useAuth } from '../store/auth'

const SERVER_KEY = 'khata_server_url'

function getBaseUrl(): string {
  try {
    const saved = localStorage.getItem(SERVER_KEY)
    if (saved && saved.trim()) {
      return saved.trim().replace(/\/+$/, '') + '/api'
    }
  } catch { /* */ }
  return '/api'
}

export function getServerUrl(): string {
  try {
    return localStorage.getItem(SERVER_KEY) || ''
  } catch { return '' }
}

export function setServerUrl(url: string) {
  try {
    const normalized = url.trim().replace(/\/+$/, '')
    if (normalized) {
      localStorage.setItem(SERVER_KEY, normalized)
    } else {
      localStorage.removeItem(SERVER_KEY)
    }
  } catch { /* */ }
}

export const api = axios.create({ baseURL: getBaseUrl(), withCredentials: true })

// Update base URL when server changes
export function refreshApiBaseUrl() {
  api.defaults.baseURL = getBaseUrl()
}

// Handle 401 without full page reload
let hasLoggedOut = false
api.interceptors.response.use(
  r => r,
  err => {
    if (err.response?.status === 401 && !hasLoggedOut) {
      const { user } = useAuth.getState()
      // Only trigger logout if user was previously authenticated
      // (prevents reload loop when app starts with no session)
      if (user) {
        hasLoggedOut = true
        useAuth.getState().logout()
        // Reset flag after navigation settles
        setTimeout(() => { hasLoggedOut = false }, 1000)
      }
    }
    return Promise.reject(err)
  }
)
