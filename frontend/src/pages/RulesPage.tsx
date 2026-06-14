import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, Sparkles } from 'lucide-react'
import { Screen, Card, CardBody, ListRow, Field, Chip, Button, EmptyState } from '../components/shared'

interface Rule {
  id: string; pattern: string; category: string
}

export function RulesPage() {
  const [rules, setRules] = useState<Rule[]>([])
  const [pattern, setPattern] = useState('')
  const [category, setCategory] = useState('')
  const [loading, setLoading] = useState(false)
  const [applying, setApplying] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  const load = () => api.get<Rule[]>('/rules').then(r => setRules(r.data)).catch(() => {})
  useEffect(() => { load() }, [])

  const add = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true); setError(''); setSuccess('')
    try {
      await api.post('/rules', { pattern, category })
      setPattern(''); setCategory('')
      setSuccess(`Rule created — applied to matching transactions`)
      await load()
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setError(e.response?.data?.error ?? 'Failed')
    } finally { setLoading(false) }
  }

  const del = async (id: string) => {
    try { await api.delete(`/rules/${id}`); await load() }
    catch { setError('Failed to delete') }
  }

  const applyAll = async () => {
    setApplying(true); setError(''); setSuccess('')
    try {
      const { data } = await api.post<{ message: string }>('/rules/apply')
      setSuccess(data.message)
    } catch { setError('Failed to apply rules') }
    finally { setApplying(false) }
  }

  const applyAllBtn = (
    <Button variant="secondary" size="sm" onClick={applyAll} disabled={applying}>
      <Sparkles size={14} /> {applying ? 'Applying…' : 'Apply All'}
    </Button>
  )

  return (
    <Screen title="Category Rules" subtitle="Auto-categorize transactions by payee keywords" actions={applyAllBtn}>
      <div style={{ marginBottom: 20 }}>
        <Card>
          <CardBody>
            <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              <Field style={{ flex: 1, minWidth: 120 }} placeholder="Keyword (e.g. ZOMATO)" value={pattern} onChange={e => setPattern(e.target.value)} required />
              <Field style={{ flex: 1, minWidth: 120 }} placeholder="Category (e.g. Food & Dining)" value={category} onChange={e => setCategory(e.target.value)} required />
              <Button disabled={loading}><Plus size={15} /> Add Rule</Button>
            </form>
            {error && <p className="text-error" style={{ marginTop: 12 }}>{error}</p>}
            {success && <p className="text-success" style={{ marginTop: 12 }}>{success}</p>}
          </CardBody>
        </Card>
      </div>

      <Card>
        {rules.length === 0 && <EmptyState icon="🔤" title="No rules yet" description="Auto-categorize transactions by keyword. Add rules like ZOMATO → Food & Dining to save time." action={{ label: 'Create your first rule', onClick: () => document.querySelector<HTMLInputElement>('input[placeholder*="keyword"]')?.focus() }} />}
        {rules.map(r => (
          <ListRow
            key={r.id}
            trailing={<Button variant="ghost" size="sm" style={{ color: 'var(--expense)' }} onClick={() => del(r.id)}><Trash2 size={14} /></Button>}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, overflow: 'hidden' }}>
              <Chip color="gray" title={r.pattern}>{r.pattern}</Chip>
              <span style={{ color: 'var(--text-2)', flexShrink: 0 }}>→</span>
              <Chip color="purple">{r.category}</Chip>
            </div>
          </ListRow>
        ))}
      </Card>
    </Screen>
  )
}
