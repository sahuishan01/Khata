import { useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { useAuth } from '../store/auth'
import { Eye, EyeOff } from 'lucide-react'

export function ResetPasswordPage() {
  const location = useLocation()
  const forceReset = (location.state as { forceReset?: boolean })?.forceReset
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')
  const [loading, setLoading] = useState(false)
  const [showCurrentPwd, setShowCurrentPwd] = useState(false)
  const [showNewPwd, setShowNewPwd] = useState(false)
  const [showConfirmPwd, setShowConfirmPwd] = useState(false)
  const resetPassword = useAuth(s => s.resetPassword)
  const navigate = useNavigate()

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (newPassword !== confirmPassword) {
      setError('Passwords do not match')
      return
    }
    setLoading(true)
    setError('')
    setSuccess('')
    try {
      await resetPassword(currentPassword, newPassword)
      setSuccess('Password updated successfully')
      setTimeout(() => navigate('/'), 1500)
    } catch (err: unknown) {
      const e = err as { response?: { status?: number; data?: { error?: string } }; message?: string }
      if (e.response?.status === 401) {
        setError('Current password is incorrect')
      } else {
        setError(e.response?.data?.error ?? e.message ?? 'Failed to reset password')
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="auth-wrapper">
      <div className="auth-card">
        <div className="auth-logo">
          <div className="sidebar-logo" style={{ width: 40, height: 40, fontSize: 18 }}>₹</div>
          <span style={{ fontSize: 22, fontWeight: 700, color: 'var(--text)', letterSpacing: '-0.4px' }}>Khata</span>
        </div>

        <h2 style={{ fontSize: 18, textAlign: 'center', marginBottom: 6 }}>Reset Password</h2>
        <p className="text-muted" style={{ textAlign: 'center', marginBottom: 24 }}>Enter your current password and a new one</p>

        {forceReset && (
          <p className="text-warning" style={{ textAlign: 'center', marginBottom: 16, padding: '8px 12px', background: 'var(--warning-bg, #fff3cd)', borderRadius: 8 }}>
            You must reset your password before continuing
          </p>
        )}

        <form onSubmit={submit}>
          <div className="form-group" style={{ position: 'relative' }}>
            <label className="form-label">Current Password</label>
            <input
              type={showCurrentPwd ? 'text' : 'password'}
              className="form-input"
              placeholder="••••••••"
              value={currentPassword}
              onChange={e => setCurrentPassword(e.target.value)}
              required
              autoComplete="current-password"
              style={{ paddingRight: 36 }}
            />
            <button type="button" onClick={() => setShowCurrentPwd(!showCurrentPwd)} style={{ position: 'absolute', right: 8, top: '50%', transform: 'translateY(-50%)', background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-2)', padding: 4, lineHeight: 1 }}>
              {showCurrentPwd ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>
          <div className="form-group" style={{ position: 'relative' }}>
            <label className="form-label">New Password</label>
            <input
              type={showNewPwd ? 'text' : 'password'}
              className="form-input"
              placeholder="Min. 12 characters"
              value={newPassword}
              onChange={e => setNewPassword(e.target.value)}
              required
              minLength={12}
              autoComplete="new-password"
              style={{ paddingRight: 36 }}
            />
            <button type="button" onClick={() => setShowNewPwd(!showNewPwd)} style={{ position: 'absolute', right: 8, top: '50%', transform: 'translateY(-50%)', background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-2)', padding: 4, lineHeight: 1 }}>
              {showNewPwd ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>
          <div className="form-group" style={{ position: 'relative' }}>
            <label className="form-label">Confirm New Password</label>
            <input
              type={showConfirmPwd ? 'text' : 'password'}
              className="form-input"
              placeholder="Repeat new password"
              value={confirmPassword}
              onChange={e => setConfirmPassword(e.target.value)}
              required
              minLength={12}
              autoComplete="new-password"
              style={{ paddingRight: 36 }}
            />
            <button type="button" onClick={() => setShowConfirmPwd(!showConfirmPwd)} style={{ position: 'absolute', right: 8, top: '50%', transform: 'translateY(-50%)', background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-2)', padding: 4, lineHeight: 1 }}>
              {showConfirmPwd ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>

          {error && <p className="text-error mb-3">{error}</p>}
          {success && <p className="text-success mb-3">{success}</p>}

          <button type="submit" className="btn btn-primary btn-full btn-lg" disabled={loading}>
            {loading ? 'Resetting…' : 'Reset Password'}
          </button>
        </form>

        <p className="text-muted" style={{ textAlign: 'center', marginTop: 20, fontSize: 13 }}>
          <a href="/" onClick={e => { e.preventDefault(); navigate(-1) }} style={{ fontWeight: 500 }}>Go back</a>
        </p>
      </div>
    </div>
  )
}
