import { create } from 'zustand'

interface PrivacyState {
  blurMode: boolean
  toggleBlur: () => void
}

export const usePrivacy = create<PrivacyState>(set => ({
  blurMode: true,
  toggleBlur: () => set(s => ({ blurMode: !s.blurMode })),
}))
