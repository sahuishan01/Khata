import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, Sparkles } from 'lucide-react'

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

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
        <h1 className="page-title">Category Rules</h1>
        <button className="btn btn-secondary btn-sm" onClick={applyAll} disabled={applying}>
          <Sparkles size={14} /> {applying ? 'Applying…' : 'Apply All'}
        </button>
      </div>
      <p className="text-muted" style={{ marginBottom: 20 }}>Auto-categorize transactions by payee keywords</p>

      <div className="card" style={{ marginBottom: 20 }}>
        <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
          <input className="form-input" style={{ flex: 1, minWidth: 120 }} placeholder="Keyword (e.g. ZOMATO)" value={pattern} onChange={e => setPattern(e.target.value)} required />
          <input className="form-input" style={{ flex: 1, minWidth: 120 }} placeholder="Category (e.g. Food & Dining)" value={category} onChange={e => setCategory(e.target.value)} required />
          <button className="btn btn-primary" disabled={loading}><Plus size={15} /> Add Rule</button>
        </form>
        {error && <p className="text-error mt-3">{error}</p>}
        {success && <p className="text-success mt-3">{success}</p>}
      </div>

      <div className="card" style={{ padding: 0 }}>
        {rules.length === 0 && <p className="text-muted" style={{ padding: 20, textAlign: 'center' }}>No rules yet. Add keywords like ZOMATO → Food & Dining</p>}
        {rules.map(r => (
          <div key={r.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderBottom: '1px solid var(--border)' }}>
            <span className="badge badge-gray" style={{ fontFamily: 'monospace', fontSize: 12 }}>{r.pattern}</span>
            <span style={{ color: 'var(--text-2)' }}>→</span>
            <span className="badge badge-purple">{r.category}</span>
            <div style={{ flex: 1 }} />
            <button className="btn btn-ghost btn-sm" style={{ color: 'var(--red)' }} onClick={() => del(r.id)}><Trash2 size={14} /></button>
          </div>
        ))}
      </div>
    </div>
  )
}
