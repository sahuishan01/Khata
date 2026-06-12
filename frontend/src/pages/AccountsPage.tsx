import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, Landmark } from 'lucide-react'

interface Account {
  id: string; label: string; identifier: string
}

export function AccountsPage() {
  const [accounts, setAccounts] = useState<Account[]>([])
  const [label, setLabel] = useState('')
  const [identifier, setIdentifier] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const load = () => api.get<Account[]>('/accounts').then(r => setAccounts(r.data)).catch(() => {})
  useEffect(() => { load() }, [])

  const add = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true); setError('')
    try {
      await api.post('/accounts', { label, identifier })
      setLabel(''); setIdentifier('')
      await load()
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setError(e.response?.data?.error ?? 'Failed')
    } finally { setLoading(false) }
  }

  const del = async (id: string) => {
    try { await api.delete(`/accounts/${id}`); await load() }
    catch { setError('Failed to delete') }
  }

  return (
    <div>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Your Accounts</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Mark accounts as your own so transfers between them don't count as income/expense</p>

      <div className="card" style={{ marginBottom: 20 }}>
        <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
          <input className="form-input" style={{ flex: 1, minWidth: 140 }} placeholder="Label (e.g. My Salary)" value={label} onChange={e => setLabel(e.target.value)} required />
          <input className="form-input" style={{ flex: 1, minWidth: 160 }} placeholder="UPI / Account / Name" value={identifier} onChange={e => setIdentifier(e.target.value)} required />
          <button className="btn btn-primary" disabled={loading}><Plus size={15} /> Add</button>
        </form>
        {error && <p className="text-error mt-3">{error}</p>}
      </div>

      <div className="card" style={{ padding: 0 }}>
        {accounts.length === 0 && <p className="text-muted" style={{ padding: 20, textAlign: 'center' }}>No accounts added yet</p>}
        {accounts.map(a => (
          <div key={a.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderBottom: '1px solid var(--border)' }}>
            <div className="stat-icon" style={{ background: 'var(--accent-dim)', color: 'var(--accent-text)' }}><Landmark size={15} /></div>
            <div style={{ flex: 1 }}>
              <div style={{ fontWeight: 600, fontSize: 13.5, color: 'var(--text-heading)' }}>{a.label}</div>
              <div className="text-muted" style={{ fontSize: 12 }}>{a.identifier}</div>
            </div>
            <button className="btn btn-ghost btn-sm" style={{ color: 'var(--red)' }} onClick={() => del(a.id)}><Trash2 size={14} /></button>
          </div>
        ))}
      </div>
    </div>
  )
}
