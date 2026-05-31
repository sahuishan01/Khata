import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../store/auth'

export function LoginPage() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const login = useAuth(s => s.login)
  const navigate = useNavigate()

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await login(email, password)
      navigate('/')
    } catch {
      setError('Invalid email or password')
    }
  }

  return (
    <div style={{ maxWidth: 400, margin: '80px auto', padding: 24 }}>
      <h2>Sign in to Khata</h2>
      <form onSubmit={submit}>
        <input type="email" placeholder="Email" value={email}
          onChange={e => setEmail(e.target.value)} required
          style={{ display: 'block', width: '100%', marginBottom: 12, padding: 8, boxSizing: 'border-box' }} />
        <input type="password" placeholder="Password" value={password}
          onChange={e => setPassword(e.target.value)} required
          style={{ display: 'block', width: '100%', marginBottom: 12, padding: 8, boxSizing: 'border-box' }} />
        {error && <p style={{ color: 'red' }}>{error}</p>}
        <button type="submit" style={{ width: '100%', padding: 10 }}>Login</button>
      </form>
      <p><Link to="/register">No account? Register</Link></p>
    </div>
  )
}
