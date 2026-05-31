import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../store/auth'

export function RegisterPage() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const register = useAuth(s => s.register)
  const navigate = useNavigate()

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await register(email, password)
      navigate('/')
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } }; message?: string }
      if (e.response?.data?.error) {
        setError(e.response.data.error)
      } else if (e.message === 'Network Error') {
        setError('Cannot reach server — make sure the backend is running')
      } else {
        setError(e.message ?? 'Registration failed')
      }
    }
  }

  return (
    <div style={{ maxWidth: 400, margin: '80px auto', padding: 24 }}>
      <h2>Create Khata account</h2>
      <form onSubmit={submit}>
        <input type="email" placeholder="Email" value={email}
          onChange={e => setEmail(e.target.value)} required
          style={{ display: 'block', width: '100%', marginBottom: 12, padding: 8, boxSizing: 'border-box' }} />
        <input type="password" placeholder="Password (min 8 chars)" value={password}
          onChange={e => setPassword(e.target.value)} required minLength={8}
          style={{ display: 'block', width: '100%', marginBottom: 12, padding: 8, boxSizing: 'border-box' }} />
        {error && <p style={{ color: 'red' }}>{error}</p>}
        <button type="submit" style={{ width: '100%', padding: 10 }}>Register</button>
      </form>
      <p><Link to="/login">Already have an account?</Link></p>
    </div>
  )
}
