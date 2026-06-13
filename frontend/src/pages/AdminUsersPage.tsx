import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { EmptyState } from '../components/EmptyState'

interface User {
  id: string
  email: string
  role: string
}

export function AdminUsersPage() {
  const [users, setUsers] = useState<User[]>([])
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')
  const [loading, setLoading] = useState(false)
  const [deleting, setDeleting] = useState<string | null>(null)

  const loadUsers = async () => {
    try {
      const { data } = await api.get<User[]>('/auth/users')
      setUsers(data)
    } catch {
      setError('Failed to load users')
    }
  }

  useEffect(() => { loadUsers() }, [])

  const createUser = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    setError('')
    setSuccess('')
    try {
      await api.post('/auth/users', { email, password })
      setSuccess(`User ${email} created`)
      setEmail('')
      setPassword('')
      await loadUsers()
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setError(e.response?.data?.error ?? 'Failed to create user')
    } finally {
      setLoading(false)
    }
  }

  const deleteUser = async (id: string, email: string) => {
    if (!confirm(`Delete user ${email}? This cannot be undone.`)) return
    setDeleting(id)
    setError('')
    setSuccess('')
    try {
      await api.delete(`/auth/users/${id}`)
      setSuccess(`User ${email} deleted`)
      await loadUsers()
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setError(e.response?.data?.error ?? 'Failed to delete user')
    } finally {
      setDeleting(null)
    }
  }

  return (
    <div>
      <h1 style={{ fontSize: 20, fontWeight: 700, marginBottom: 4 }}>Manage Users</h1>
      <p className="text-muted" style={{ marginBottom: 24 }}>Add and manage user accounts</p>

      <div className="card" style={{ marginBottom: 24 }}>
        <h2 style={{ fontSize: 16, fontWeight: 600, marginBottom: 16 }}>Add User</h2>
        <form onSubmit={createUser}>
          <div className="form-group">
            <label className="form-label">Email</label>
            <input
              type="email"
              className="form-input"
              placeholder="user@example.com"
              value={email}
              onChange={e => setEmail(e.target.value)}
              required
            />
          </div>
          <div className="form-group">
            <label className="form-label">Password</label>
            <input
              type="password"
              className="form-input"
              placeholder="Min. 8 characters"
              value={password}
              onChange={e => setPassword(e.target.value)}
              required
              minLength={8}
            />
          </div>

          {error && <p className="text-error mb-3">{error}</p>}
          {success && <p className="text-success mb-3">{success}</p>}

          <button type="submit" className="btn btn-primary" disabled={loading}>
            {loading ? 'Adding…' : 'Add User'}
          </button>
        </form>
      </div>

      <div className="card">
        <h2 style={{ fontSize: 16, fontWeight: 600, marginBottom: 16 }}>Users</h2>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid var(--hairline)', textAlign: 'left' }}>
              <th style={{ padding: '8px 12px', fontWeight: 600, fontSize: 12, textTransform: 'uppercase', color: 'var(--text-muted)' }}>Email</th>
              <th style={{ padding: '8px 12px', fontWeight: 600, fontSize: 12, textTransform: 'uppercase', color: 'var(--text-muted)' }}>Role</th>
              <th style={{ padding: '8px 12px', fontWeight: 600, fontSize: 12, textTransform: 'uppercase', color: 'var(--text-muted)' }}></th>
            </tr>
          </thead>
          <tbody>
            {users.map(u => (
              <tr key={u.id} style={{ borderBottom: '1px solid var(--hairline)' }}>
                <td style={{ padding: '10px 12px' }}>{u.email}</td>
                <td style={{ padding: '10px 12px' }}>
                  <span style={{
                    display: 'inline-block',
                    padding: '2px 8px',
                    borderRadius: 4,
                    fontSize: 12,
                    fontWeight: 600,
                    background: u.role === 'admin' ? 'var(--primary-light)' : 'var(--bg-subtle)',
                    color: u.role === 'admin' ? 'var(--primary)' : 'var(--text-muted)',
                  }}>
                    {u.role}
                  </span>
                </td>
                <td style={{ padding: '10px 12px', textAlign: 'right' }}>
                  <button
                    className="btn btn-ghost btn-sm"
                    style={{ color: 'var(--danger, #e53e3e)' }}
                    onClick={() => deleteUser(u.id, u.email)}
                    disabled={deleting === u.id}
                  >
                    {deleting === u.id ? '…' : 'Delete'}
                  </button>
                </td>
              </tr>
            ))}
            {users.length === 0 && (
              <tr><td colSpan={3} style={{ padding: 20 }}><EmptyState icon="👥" title="No users yet" description="Add users to your organization so they can access shared financial data." action={{ label: 'Add user', onClick: () => document.querySelector<HTMLInputElement>('input[type="email"]')?.focus() }} /></td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  )
}
