import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, Landmark, Eye, EyeOff } from 'lucide-react'
import { usePrivacy } from '../store/privacy'
import { maskIdentifier } from '../utils/pii'
import { Screen, Card, CardBody, ListRow, ListRowText, Field, Button, EmptyState } from '../components/shared'

interface Account {
  id: string; label: string; identifier: string
}

export function AccountsPage() {
  const [accounts, setAccounts] = useState<Account[]>([])
  const [label, setLabel] = useState('')
  const [identifier, setIdentifier] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const { blurMode, toggleBlur } = usePrivacy()

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

  const blurToggle = (
    <Button variant="ghost" size="sm" onClick={toggleBlur}>
      {blurMode ? <EyeOff size={14} /> : <Eye size={14} />}
      <span style={{ fontSize: 11, marginLeft: 4 }}>{blurMode ? 'Blurred' : 'Visible'}</span>
    </Button>
  )

  return (
    <Screen title="Your Accounts" subtitle="Mark accounts as your own so transfers between them don't count as income/expense" actions={blurToggle}>
      <div style={{ marginBottom: 20 }}>
        <Card>
          <CardBody>
            <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              <Field style={{ flex: 1, minWidth: 140 }} placeholder="Label (e.g. My Salary)" value={label} onChange={e => setLabel(e.target.value)} required />
              <Field style={{ flex: 1, minWidth: 160 }} placeholder="UPI / Account / Name" value={identifier} onChange={e => setIdentifier(e.target.value)} required />
              <Button disabled={loading}><Plus size={15} /> Add</Button>
            </form>
            {error && <p className="text-error" style={{ marginTop: 12 }}>{error}</p>}
          </CardBody>
        </Card>
      </div>

      <Card>
        {accounts.length === 0 && <EmptyState icon="🏦" title="No accounts yet" description="Add your bank accounts, UPI IDs, or wallets so transfers between them don't count as income or expense." action={{ label: 'Add your first account', onClick: () => document.querySelector<HTMLInputElement>('input[placeholder*="Label"]')?.focus() }} />}
        {accounts.map(a => (
          <ListRow
            key={a.id}
            leading={<div style={{ width: 32, height: 32, borderRadius: 8, background: 'var(--brand-soft)', color: 'var(--brand)', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}><Landmark size={15} /></div>}
            trailing={<Button variant="ghost" size="sm" style={{ color: 'var(--expense)' }} onClick={() => del(a.id)}><Trash2 size={14} /></Button>}
          >
            <ListRowText primary={a.label} secondary={blurMode ? maskIdentifier(a.identifier) : a.identifier} />
          </ListRow>
        ))}
      </Card>
    </Screen>
  )
}
