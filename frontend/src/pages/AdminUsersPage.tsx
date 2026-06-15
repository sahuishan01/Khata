import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Screen, Card, CardBody, ListRow, ListRowText, Field, Button, Chip, EmptyState } from '../components/shared'
import { Eye, EyeOff } from 'lucide-react'

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
  const [showPwd, setShowPwd] = useState(false)
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
    <Screen title="Manage Users">
      <div style={{ marginBottom: 24 }}>
        <Card>
          <CardBody>
            <form onSubmit={createUser} style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <Field type="email" placeholder="user@example.com" value={email} onChange={e => setEmail(e.target.value)} required />
              <div style={{ position: 'relative' }}>
                <input type={showPwd ? 'text' : 'password'} className="form-input" placeholder="Min. 8 characters" value={password} onChange={e => setPassword(e.target.value)} required minLength={8} style={{ paddingRight: 36, width: '100%' }} />
                <button type="button" onClick={() => setShowPwd(!showPwd)} style={{ position: 'absolute', right: 8, top: '50%', transform: 'translateY(-50%)', background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-2)', padding: 4, lineHeight: 1 }}>
                  {showPwd ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
              {error && <p style={{ fontSize: 13, color: 'var(--expense)' }}>{error}</p>}
              {success && <p style={{ fontSize: 13, color: 'var(--income)' }}>{success}</p>}
              <Button disabled={loading}>{loading ? 'Adding…' : 'Add User'}</Button>
            </form>
          </CardBody>
        </Card>
      </div>

      <Card>
        {users.length === 0 && <EmptyState icon="👥" title="No users yet" description="Add users to your organization so they can access shared financial data." action={{ label: 'Add user', onClick: () => document.querySelector<HTMLInputElement>('input[type="email"]')?.focus() }} />}
        {users.map(u => (
          <ListRow
            key={u.id}
            leading={<Chip color={u.role === 'admin' ? 'purple' : 'gray'}>{u.role}</Chip>}
            trailing={
              <Button variant="danger" size="sm" onClick={() => deleteUser(u.id, u.email)} disabled={deleting === u.id}>
                {deleting === u.id ? '…' : 'Delete'}
              </Button>
            }
          >
            <ListRowText primary={u.email} />
          </ListRow>
        ))}
      </Card>
    </Screen>
  )
}
