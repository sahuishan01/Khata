import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../store/auth'

export function ResetPasswordPage() {
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')
  const [loading, setLoading] = useState(false)
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

        <form onSubmit={submit}>
          <div className="form-group">
            <label className="form-label">Current Password</label>
            <input
              type="password"
              className="form-input"
              placeholder="••••••••"
              value={currentPassword}
              onChange={e => setCurrentPassword(e.target.value)}
              required
              autoComplete="current-password"
            />
          </div>
          <div className="form-group">
            <label className="form-label">New Password</label>
            <input
              type="password"
              className="form-input"
              placeholder="Min. 8 characters"
              value={newPassword}
              onChange={e => setNewPassword(e.target.value)}
              required
              minLength={8}
              autoComplete="new-password"
            />
          </div>
          <div className="form-group">
            <label className="form-label">Confirm New Password</label>
            <input
              type="password"
              className="form-input"
              placeholder="Repeat new password"
              value={confirmPassword}
              onChange={e => setConfirmPassword(e.target.value)}
              required
              minLength={8}
              autoComplete="new-password"
            />
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
