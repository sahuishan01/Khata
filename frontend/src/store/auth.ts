import { create } from 'zustand'
import { api } from '../api/client'

interface UserInfo {
  id: string
  email: string
  role: string
}

interface AuthState {
  user: UserInfo | null
  loading: boolean
  mustResetPassword: boolean
  login: (email: string, password: string) => Promise<void>
  logout: () => void
  setup: (email: string, password: string) => Promise<void>
  adminCreateUser: (email: string, password: string) => Promise<void>
  resetPassword: (currentPassword: string, newPassword: string) => Promise<void>
  fetchMe: () => Promise<void>
  checkSetupStatus: () => Promise<boolean>
}

export const useAuth = create<AuthState>((set, get) => ({
  user: null,
  loading: true,
  mustResetPassword: false,

  login: async (email, password) => {
    const { data } = await api.post<{ token: string; must_reset_password: boolean }>('/auth/login', { email, password })
    set({ mustResetPassword: data.must_reset_password })
    if (!data.must_reset_password) {
      await get().fetchMe()
    }
  },

  logout: async () => {
    try { await api.post('/auth/logout') } catch { /* ignore */ }
    set({ user: null, mustResetPassword: false })
  },

  setup: async (email, password) => {
    await api.post<{ token: string }>('/auth/setup', { email, password })
    await get().fetchMe()
  },

  adminCreateUser: async (email, password) => {
    await api.post('/auth/users', { email, password })
  },

  resetPassword: async (currentPassword, newPassword) => {
    await api.post('/auth/reset-password', { current_password: currentPassword, new_password: newPassword })
  },

  fetchMe: async () => {
    try {
      const { data } = await api.get<UserInfo>('/auth/me')
      set({ user: data, loading: false })
    } catch {
      set({ user: null, loading: false })
    }
  },

  checkSetupStatus: async () => {
    const { data } = await api.get<{ setup_required: boolean }>('/auth/setup-status')
    return data.setup_required
  },
}))

// Bootstrap: try to load current user from session cookie
useAuth.getState().fetchMe()
