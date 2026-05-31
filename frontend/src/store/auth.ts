import { create } from 'zustand'
import { api } from '../api/client'

interface AuthState {
  token: string | null
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string) => Promise<void>
  logout: () => void
}

export const useAuth = create<AuthState>(set => ({
  token: localStorage.getItem('token'),
  login: async (email, password) => {
    const { data } = await api.post<{ token: string }>('/auth/login', { email, password })
    localStorage.setItem('token', data.token)
    set({ token: data.token })
  },
  register: async (email, password) => {
    const { data } = await api.post<{ token: string }>('/auth/register', { email, password })
    localStorage.setItem('token', data.token)
    set({ token: data.token })
  },
  logout: () => {
    localStorage.removeItem('token')
    set({ token: null })
  }
}))
