import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { useAuth } from '../store/auth'
import { Sun, Moon, Trash2, KeyRound, User, LogOut, Check, AlertTriangle } from 'lucide-react'
import { Screen, Card, CardBody, ListRow, ListRowText, Button, Field } from '../components/shared'

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
      <Screen title="Settings" subtitle="Manage your account">
        {msg && (
          <div style={{
            background: msgType === 'error' ? 'var(--expense-soft)' : 'var(--income-soft)',
            borderRadius: 8, padding: '10px 14px', fontSize: 13, marginBottom: 16,
          }}>
            {msg}
          </div>
        )}

        <div style={{ marginBottom: 16 }}>
          <Card>
            <CardBody>
              <ListRow
                leading={<div style={{ width: 32, height: 32, borderRadius: 8, background: 'var(--brand-soft)', color: 'var(--brand)', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}><User size={16} /></div>}
                trailing={<Button variant="secondary" size="sm" onClick={() => { setNewEmail(user?.email || ''); setShowEmailDialog(true) }}>Change</Button>}
              >
                <ListRowText primary={user?.email || 'Unknown'} secondary={user?.role} />
              </ListRow>
            </CardBody>
          </Card>
        </div>

        <div style={{ marginBottom: 16 }}>
          <Card>
            <CardBody>
              <ListRow
                leading={isDark ? <Moon size={16} /> : <Sun size={16} style={{ color: 'var(--warn)' }} />}
                trailing={<Button variant={isDark ? 'primary' : 'secondary'} size="sm" onClick={toggleTheme}>{isDark ? 'On' : 'Off'}</Button>}
              >
                <ListRowText primary="Dark Mode" />
              </ListRow>
            </CardBody>
          </Card>
        </div>

        <div style={{ marginBottom: 16 }}>
          <Card>
            <CardBody style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              <Button variant="ghost" onClick={() => navigate('/reset-password')}>
                <KeyRound size={15} /> Reset Password
              </Button>
              <Button variant="ghost" style={{ color: 'var(--expense)' }} onClick={() => setShowClearDialog(true)}>
                <Trash2 size={15} /> Clear All Data…
              </Button>
              <Button variant="ghost" onClick={() => { logout(); navigate('/login') }}>
                <LogOut size={15} /> Logout
              </Button>
            </CardBody>
          </Card>
        </div>
      </Screen>

      {showClearDialog && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20 }}>
          <div style={{ maxWidth: 380, width: '100%' }}>
            <Card>
              <CardBody>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                  <AlertTriangle size={18} style={{ color: 'var(--expense)' }} />
                  <h3 style={{ fontSize: 16, fontWeight: 600, color: 'var(--expense)' }}>Clear All Data</h3>
                </div>
                <p style={{ color: 'var(--text-2)', marginBottom: 12, fontSize: 13 }}>This will permanently delete all transactions, statements, and chat history. This action cannot be undone.</p>
                <p style={{ fontSize: 13, color: 'var(--text)', marginBottom: 8 }}>Type <strong>DELETE</strong> to confirm:</p>
                <Field value={clearConfirmText} onChange={e => setClearConfirmText(e.target.value)} placeholder="Type DELETE" autoFocus />
                <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 12 }}>
                  <Button variant="secondary" onClick={() => { setShowClearDialog(false); setClearConfirmText('') }}>Cancel</Button>
                  <Button variant="danger" disabled={clearConfirmText !== 'DELETE'} onClick={() => { clearAllData(); setClearConfirmText('') }}>Clear Everything</Button>
                </div>
              </CardBody>
            </Card>
          </div>
        </div>
      )}

      {showEmailDialog && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20 }}>
          <div style={{ maxWidth: 380, width: '100%' }}>
            <Card>
              <CardBody>
                <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 12 }}>Change Email</h3>
                <Field value={newEmail} onChange={e => setNewEmail(e.target.value)} placeholder="New email address" />
                <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 12 }}>
                  <Button variant="secondary" onClick={() => setShowEmailDialog(false)}>Cancel</Button>
                  <Button variant="primary" onClick={updateEmail}><Check size={14} /> Save</Button>
                </div>
              </CardBody>
            </Card>
          </div>
        </div>
      )}
    </div>
  )
}
