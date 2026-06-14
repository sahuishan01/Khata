import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, AlertTriangle } from 'lucide-react'
import { Screen, Card, CardBody, Field, Chip, Button, ProgressBar, Amount, EmptyState } from '../components/shared'

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
    <Screen title="Budgets" subtitle="Set monthly spending limits per category">
      <div style={{ marginBottom: 20 }}>
        <Card>
          <CardBody>
            <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              <Field style={{ flex: 1, minWidth: 120 }} placeholder="Category" value={category} onChange={e => setCategory(e.target.value)} required />
              <Field style={{ flex: 1, minWidth: 120 }} type="number" step="0.01" placeholder="Monthly limit" value={monthlyLimit} onChange={e => setMonthlyLimit(e.target.value)} required />
              <Button disabled={loading}><Plus size={15} /> Set Budget</Button>
            </form>
            {error && <p className="text-error" style={{ marginTop: 12 }}>{error}</p>}
          </CardBody>
        </Card>
      </div>

      {budgets.map(b => {
        const s = getStatus(b.category)
        const pct = s?.pct ?? 0
        return (
          <div key={b.id} style={{ marginBottom: 10 }}>
            <Card>
              <CardBody>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
                  <div>
                    <Chip color="purple">{b.category}</Chip>
                    <span style={{ fontSize: 12, color: 'var(--text-2)', marginLeft: 8 }}>
                      <Amount paise={b.monthly_limit} size="sm" /> / month
                    </span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    {pct >= 80 && <AlertTriangle size={15} style={{ color: pct >= 100 ? 'var(--expense)' : 'var(--warn)' }} />}
                    <span style={{ fontWeight: 700, fontSize: 14, color: pct >= 100 ? 'var(--expense)' : pct >= 80 ? 'var(--warn)' : 'var(--income)' }}>{pct.toFixed(0)}%</span>
                    <Button variant="ghost" size="sm" style={{ color: 'var(--expense)' }} onClick={() => del(b.id)}><Trash2 size={14} /></Button>
                  </div>
                </div>
                <ProgressBar pct={pct} />
                {s && <div style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>Spent: <Amount paise={s.spent} size="sm" /> / <Amount paise={b.monthly_limit} size="sm" /></div>}
              </CardBody>
            </Card>
          </div>
        )
      })}
      {budgets.length === 0 && <EmptyState icon="🎯" title="No budgets yet" description="Set a monthly limit per category to get alerts before you overspend." action={{ label: 'Set your first budget', onClick: () => document.querySelector<HTMLInputElement>('input[placeholder*="Category"]')?.focus() }} />}
    </Screen>
  )
}
