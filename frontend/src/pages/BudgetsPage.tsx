import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, AlertTriangle } from 'lucide-react'

interface Budget { id: string; category: string; monthly_limit: number }
interface BudgetStatus { category: string; monthly_limit: number; spent: number; pct: number }

export function BudgetsPage() {
  const [budgets, setBudgets] = useState<Budget[]>([])
  const [status, setStatus] = useState<BudgetStatus[]>([])
  const [category, setCategory] = useState('')
  const [monthlyLimit, setMonthlyLimit] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const load = async () => {
    try {
      const [b, s] = await Promise.all([
        api.get<Budget[]>('/budgets'),
        api.get<BudgetStatus[]>('/budgets/status'),
      ])
      setBudgets(b.data); setStatus(s.data)
    } catch { setError('Failed to load') }
  }
  useEffect(() => { load() }, [])

  const add = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true); setError('')
    try {
      await api.post('/budgets', { category, monthly_limit: parseFloat(monthlyLimit) })
      setCategory(''); setMonthlyLimit('')
      await load()
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setError(e.response?.data?.error ?? 'Failed')
    } finally { setLoading(false) }
  }

  const del = async (id: string) => {
    try { await api.delete(`/budgets/${id}`); await load() }
    catch { setError('Failed') }
  }

  const getStatus = (cat: string) => status.find(s => s.category === cat)

  return (
    <div>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Budgets</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Set monthly spending limits per category</p>

      <div className="card" style={{ marginBottom: 20 }}>
        <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
          <input className="form-input" style={{ flex: 1, minWidth: 120 }} placeholder="Category" value={category} onChange={e => setCategory(e.target.value)} required />
          <input className="form-input" style={{ flex: 1, minWidth: 120 }} type="number" step="0.01" placeholder="Monthly limit" value={monthlyLimit} onChange={e => setMonthlyLimit(e.target.value)} required />
          <button className="btn btn-primary" disabled={loading}><Plus size={15} /> Set Budget</button>
        </form>
        {error && <p className="text-error mt-3">{error}</p>}
      </div>

      {budgets.map(b => {
        const s = getStatus(b.category)
        const pct = s?.pct ?? 0
        const barColor = pct >= 100 ? 'var(--red)' : pct >= 80 ? 'var(--amber)' : 'var(--green)'
        return (
          <div key={b.id} className="card" style={{ marginBottom: 10, padding: 14 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
              <div>
                <span className="badge badge-purple" style={{ fontSize: 12 }}>{b.category}</span>
                <span style={{ fontSize: 12, color: 'var(--text-2)', marginLeft: 8 }}>
                  ₹{b.monthly_limit.toLocaleString('en-IN')} / month
                </span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                {pct >= 80 && <AlertTriangle size={15} style={{ color: barColor }} />}
                <span style={{ fontWeight: 700, fontSize: 14, color: barColor }}>{pct.toFixed(0)}%</span>
                <button className="btn btn-ghost btn-sm" style={{ color: 'var(--red)' }} onClick={() => del(b.id)}><Trash2 size={14} /></button>
              </div>
            </div>
            <div style={{ height: 6, background: 'var(--border)', borderRadius: 3, overflow: 'hidden' }}>
              <div style={{ width: `${Math.min(pct, 100)}%`, height: '100%', background: barColor, borderRadius: 3, transition: 'width 0.3s' }} />
            </div>
            {s && <div style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>Spent: ₹{s.spent.toLocaleString('en-IN', { maximumFractionDigits: 0 })} / ₹{b.monthly_limit.toLocaleString('en-IN', { maximumFractionDigits: 0 })}</div>}
          </div>
        )
      })}
      {budgets.length === 0 && <p className="text-muted" style={{ textAlign: 'center', padding: 20 }}>No budgets set yet. Add one above.</p>}
    </div>
  )
}
