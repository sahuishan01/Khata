import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { useAuth } from '../store/auth'
import { Sun, Moon, Trash2, KeyRound, User } from 'lucide-react'

export function ProfilePage() {
  const navigate = useNavigate()
  const { user } = useAuth()
  const [isDark, setIsDark] = useState(window.matchMedia('(prefers-color-scheme: dark)').matches)
  const [showClearDialog, setShowClearDialog] = useState(false)
  const [clearResult, setClearResult] = useState('')

  useEffect(() => {
    // Check if dark mode is active by reading the CSS variable
    const isDarkMode = document.documentElement.style.colorScheme === 'dark' || document.querySelector('html')?.classList.contains('dark')
    setIsDark(!!isDarkMode)
  }, [])

  const toggleTheme = () => {
    const newDark = !isDark
    setIsDark(newDark)
    document.documentElement.style.colorScheme = newDark ? 'dark' : 'light'
    document.documentElement.classList.toggle('dark', newDark)
  }

  const clearAllData = async () => {
    try {
      await api.delete('/ingest/clear', { data: { confirm: true } })
      setClearResult('All data cleared successfully!')
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setClearResult(e.response?.data?.error ?? 'Failed to clear data')
    }
    setShowClearDialog(false)
  }

  return (
    <div style={{ maxWidth: 500, margin: '0 auto' }}>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Profile & Settings</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Manage your account and preferences</p>

      {/* Account info */}
      <div className="card" style={{ marginBottom: 16 }}>
        <h2 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Account</h2>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8 }}>
          <div className="stat-icon" style={{ background: 'var(--accent-dim)', color: 'var(--accent-text)' }}><User size={16} /></div>
          <div><div style={{ fontSize: 13, fontWeight: 500, color: 'var(--text-heading)' }}>{user?.email || 'Unknown'}</div><div className="text-muted" style={{ fontSize: 12 }}>{user?.role === 'admin' ? 'Admin' : 'User'}</div></div>
        </div>
      </div>

      {/* Preferences */}
      <div className="card" style={{ marginBottom: 16 }}>
        <h2 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Preferences</h2>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 0' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            {isDark ? <Moon size={16} style={{ color: 'var(--accent-text)' }} /> : <Sun size={16} style={{ color: 'var(--amber)' }} />}
            <span style={{ fontSize: 13 }}>Dark Mode</span>
          </div>
          <button className={`btn btn-sm ${isDark ? 'btn-primary' : 'btn-secondary'}`} onClick={toggleTheme}>{isDark ? 'On' : 'Off'}</button>
        </div>
      </div>

      {/* Actions */}
      <div className="card" style={{ marginBottom: 16 }}>
        <h2 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Account Actions</h2>
        <button className="btn btn-secondary btn-full" style={{ marginBottom: 8, justifyContent: 'flex-start', gap: 10 }} onClick={() => navigate('/reset-password')}>
          <KeyRound size={15} /> Reset Password
        </button>
        <button className="btn btn-secondary btn-full" style={{ justifyContent: 'flex-start', gap: 10, color: 'var(--red)' }} onClick={() => setShowClearDialog(true)}>
          <Trash2 size={15} /> Clear All Data
        </button>
      </div>

      {clearResult && <p className="text-success" style={{ marginBottom: 12 }}>{clearResult}</p>}

      {showClearDialog && (
        <div style={{
          position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20
        }}>
          <div className="card" style={{ maxWidth: 380, width: '100%' }}>
            <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>Clear All Data?</h3>
            <p className="text-muted" style={{ marginBottom: 16, fontSize: 13 }}>This will permanently delete all your transactions, statements, and chat history.</p>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button className="btn btn-secondary" onClick={() => setShowClearDialog(false)}>Cancel</button>
              <button className="btn btn-danger" onClick={clearAllData}>Clear Everything</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
