import axios from 'axios'
import { useAuth } from '../store/auth'

export const api = axios.create({ baseURL: '/api', withCredentials: true })

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
