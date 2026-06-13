import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { useAuth } from '../store/auth'
import { Sun, Moon, Trash2, KeyRound, User, LogOut, Check, AlertTriangle } from 'lucide-react'

export function ProfilePage() {
  const navigate = useNavigate()
  const { user, logout } = useAuth()
  const [isDark, setIsDark] = useState(false)
  const [showClearDialog, setShowClearDialog] = useState(false)
  const [clearConfirmText, setClearConfirmText] = useState('')
  const [showEmailDialog, setShowEmailDialog] = useState(false)
  const [newEmail, setNewEmail] = useState('')
  const [msg, setMsg] = useState('')
  const [msgType, setMsgType] = useState<'success' | 'error'>('success')

  useEffect(() => {
    setIsDark(document.documentElement.classList.contains('dark'))
  }, [])

  const toggleTheme = () => {
    const next = !isDark
    setIsDark(next)
    document.documentElement.classList.toggle('dark', next)
  }

  const clearAllData = async () => {
    try { await api.delete('/ingest/clear', { data: { confirm: true } }); showMsg('All data cleared!', 'success') }
    catch { showMsg('Failed', 'error') }
    setShowClearDialog(false)
  }

  const updateEmail = async () => {
    try { await api.patch('/auth/email', { email: newEmail }); showMsg('Email updated!', 'success'); setShowEmailDialog(false) }
    catch (err: unknown) { const e = err as { response?: { data?: { error?: string } } }; showMsg(e.response?.data?.error ?? 'Failed', 'error') }
  }

  const showMsg = (text: string, type: 'success' | 'error') => { setMsg(text); setMsgType(type); setTimeout(() => setMsg(''), 3000) }

  return (
    <div style={{ maxWidth: 500, margin: '0 auto' }}>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Settings</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Manage your account</p>

      {msg && <div className={`card mb-4`} style={{ background: msgType === 'error' ? 'var(--expense-soft)' : 'var(--income-soft)', border: 'none', fontSize: 13 }}>{msg}</div>}

      <div className="card" style={{ marginBottom: 16 }}>
        <h2 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Account</h2>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8 }}>
          <div className="stat-icon" style={{ background: 'var(--brand-soft)', color: 'var(--brand)' }}><User size={16} /></div>
          <div style={{ flex: 1 }}><div style={{ fontSize: 13, fontWeight: 500, color: 'var(--text)' }}>{user?.email || 'Unknown'}</div><div className="text-muted" style={{ fontSize: 12 }}>{user?.role}</div></div>
          <button className="btn btn-secondary btn-sm" onClick={() => { setNewEmail(user?.email || ''); setShowEmailDialog(true) }}>Change</button>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <h2 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Preferences</h2>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 0' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>{isDark ? <Moon size={16} /> : <Sun size={16} style={{ color: 'var(--warn)' }} />}<span style={{ fontSize: 13 }}>Dark Mode</span></div>
          <button className={`btn btn-sm ${isDark ? 'btn-primary' : 'btn-secondary'}`} onClick={toggleTheme}>{isDark ? 'On' : 'Off'}</button>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <h2 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>Actions</h2>
        <button className="btn btn-secondary btn-full" style={{ marginBottom: 8, justifyContent: 'flex-start', gap: 10 }} onClick={() => navigate('/reset-password')}><KeyRound size={15} /> Reset Password</button>
        <button className="btn btn-secondary btn-full" style={{ marginBottom: 8, justifyContent: 'flex-start', gap: 10, color: 'var(--expense)' }} onClick={() => setShowClearDialog(true)}><Trash2 size={15} /> Clear All Data…</button>
        <button className="btn btn-secondary btn-full" style={{ justifyContent: 'flex-start', gap: 10 }} onClick={() => { logout(); navigate('/login') }}><LogOut size={15} /> Logout</button>
      </div>

      {showClearDialog && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20 }}>
          <div className="card" style={{ maxWidth: 380, width: '100%', borderColor: 'var(--expense)' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <AlertTriangle size={18} style={{ color: 'var(--expense)' }} />
              <h3 style={{ fontSize: 16, fontWeight: 600, color: 'var(--expense)' }}>Clear All Data</h3>
            </div>
            <p className="text-muted" style={{ marginBottom: 12, fontSize: 13 }}>This will permanently delete all transactions, statements, and chat history. This action cannot be undone.</p>
            <p style={{ fontSize: 13, color: 'var(--text)', marginBottom: 8 }}>Type <strong>DELETE</strong> to confirm:</p>
            <input className="form-input" value={clearConfirmText} onChange={e => setClearConfirmText(e.target.value)} placeholder="Type DELETE" autoFocus />
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 12 }}>
              <button className="btn btn-secondary" onClick={() => { setShowClearDialog(false); setClearConfirmText('') }}>Cancel</button>
              <button className="btn btn-danger" disabled={clearConfirmText !== 'DELETE'} onClick={() => { clearAllData(); setClearConfirmText('') }}>Clear Everything</button>
            </div>
          </div>
        </div>
      )}

      {showEmailDialog && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20 }}>
          <div className="card" style={{ maxWidth: 380, width: '100%' }}>
            <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 12 }}>Change Email</h3>
            <input className="form-input" value={newEmail} onChange={e => setNewEmail(e.target.value)} placeholder="New email address" />
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 12 }}><button className="btn btn-secondary" onClick={() => setShowEmailDialog(false)}>Cancel</button><button className="btn btn-primary" onClick={updateEmail}><Check size={14} /> Save</button></div>
          </div>
        </div>
      )}
    </div>
  )
}
