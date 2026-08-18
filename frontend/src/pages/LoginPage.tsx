import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../store/auth'
import { getServerUrl, setServerUrl, refreshApiBaseUrl } from '../api/client'
import { ChevronDown, ChevronUp } from 'lucide-react'

export function LoginPage() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const [showPwd, setShowPwd] = useState(false)
  const [showServer, setShowServer] = useState(false)
  const [serverUrlValue, setServerUrlValue] = useState(getServerUrl())
  const login = useAuth(s => s.login)
  const mustResetPassword = useAuth(s => s.mustResetPassword)
  const navigate = useNavigate()

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    // Save server URL if changed
    if (serverUrlValue !== getServerUrl()) {
      setServerUrl(serverUrlValue)
      refreshApiBaseUrl()
    }
    setLoading(true)
    setError('')
    try {
      await login(email, password)
      if (mustResetPassword || useAuth.getState().mustResetPassword) {
        navigate('/reset-password', { state: { forceReset: true } })
      } else {
        navigate('/')
      }
    } catch (err: unknown) {
      const e = err as { response?: { status?: number }; message?: string }
      if (e.response?.status === 401) {
        setError('Invalid email or password')
      } else if (e.message === 'Network Error') {
        setError('Cannot reach server — check the server URL below')
      } else {
        setError('Login failed — please try again')
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

        <h2 style={{ fontSize: 18, textAlign: 'center', marginBottom: 6 }}>Welcome back</h2>
        <p className="text-muted" style={{ textAlign: 'center', marginBottom: 24 }}>Sign in to your account</p>

        <form onSubmit={submit}>
          <div className="form-group">
            <label className="form-label">Email</label>
            <input
              type="email"
              className="form-input"
              placeholder="you@example.com"
              value={email}
              onChange={e => setEmail(e.target.value)}
              required
              autoComplete="email"
            />
          </div>
          <div className="form-group" style={{ position: 'relative' }}>
            <label className="form-label">Password</label>
            <input
              type={showPwd ? 'text' : 'password'}
              className="form-input"
              placeholder="••••••••"
              value={password}
              onChange={e => setPassword(e.target.value)}
              required
              autoComplete="current-password"
              style={{ paddingRight: 36 }}
            />
            <button type="button" onClick={() => setShowPwd(!showPwd)} style={{ position: 'absolute', right: 8, bottom: 8, background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-2)', padding: 4, lineHeight: 1 }}>
              <span>{showPwd ? '🙈' : '👁'}</span>
            </button>
          </div>

          {error && <p className="text-error mb-3">{error}</p>}

          <button type="submit" className="btn btn-primary btn-full btn-lg" disabled={loading}>
            {loading ? 'Signing in…' : 'Sign in'}
          </button>
        </form>

        <p className="text-muted" style={{ textAlign: 'center', marginTop: 20, fontSize: 13 }}>
          <Link to="/reset-password" style={{ fontWeight: 500 }}>Forgot password?</Link>
        </p>

        {/* Server settings */}
        <div style={{ marginTop: 16, textAlign: 'center' }}>
          <button
            type="button"
            onClick={() => setShowServer(!showServer)}
            style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-muted)', fontSize: 12, display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            {showServer ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
            Server Settings
          </button>
          {showServer && (
            <div style={{ marginTop: 8, textAlign: 'left' }}>
              <label className="form-label">Server URL</label>
              <input
                type="url"
                className="form-input"
                placeholder="https://khata.algosculptor.com"
                value={serverUrlValue}
                onChange={e => setServerUrlValue(e.target.value)}
                style={{ fontSize: 13 }}
              />
              <p style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 4 }}>
                Leave empty for default (current host). For self-hosted, enter your server URL.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
