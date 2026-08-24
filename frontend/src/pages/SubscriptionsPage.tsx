import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Calendar, Plus, Trash2, CheckCircle2, RefreshCw } from 'lucide-react'
import { Screen, Card, CardBody, ListRow, ListRowText, Button, Field, Chip } from '../components/shared'

interface Subscription {
  id: string
  name: string
  amount: number
  billing_cycle: string
  next_due_date: string
  category: string
  auto_detected: boolean
  active: boolean
}

export function SubscriptionsPage() {
  const [subs, setSubs] = useState<Subscription[]>([])
  const [loading, setLoading] = useState(true)
  const [showAdd, setShowAdd] = useState(false)
  const [name, setName] = useState('')
  const [amount, setAmount] = useState('')
  const [cycle, setCycle] = useState('monthly')
  const [dueDate, setDueDate] = useState(new Date().toISOString().split('T')[0])
  const [msg, setMsg] = useState('')

  const fetchSubs = async () => {
    try {
      const res = await api.get<Subscription[]>('/subscriptions')
      setSubs(res.data)
    } catch (e) {
      console.error(e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchSubs()
  }, [])

  const handleCreate = async () => {
    if (!name || !amount) return
    try {
      await api.post('/subscriptions', {
        name,
        amount: parseFloat(amount),
        billing_cycle: cycle,
        next_due_date: dueDate,
      })
      setMsg('Subscription added!')
      setShowAdd(false)
      setName('')
      setAmount('')
      fetchSubs()
      setTimeout(() => setMsg(''), 3000)
    } catch {
      setMsg('Failed to add subscription')
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await api.delete(`/subscriptions/${id}`)
      setSubs(subs.filter(s => s.id !== id))
    } catch {
      console.error('Delete failed')
    }
  }

  const totalMonthlyCost = subs
    .filter(s => s.active)
    .reduce((acc, s) => acc + (s.billing_cycle === 'yearly' ? s.amount / 12 : s.amount), 0)

  return (
    <div style={{ maxWidth: 650, margin: '0 auto' }}>
      <Screen
        title="Subscriptions & Fixed Bills"
        subtitle="Manage recurring charges and renewal schedules"
        actions={
          <Button variant="primary" size="sm" onClick={() => setShowAdd(true)}>
            <Plus size={14} /> Add Bill
          </Button>
        }
      >
        {msg && (
          <div style={{ background: 'var(--income-soft)', padding: '10px 14px', borderRadius: 8, fontSize: 13, marginBottom: 16 }}>
            {msg}
          </div>
        )}

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginBottom: 16 }}>
          <Card>
            <CardBody>
              <div style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 4 }}>Total Monthly Commitments</div>
              <div style={{ fontSize: 22, fontWeight: 700, color: 'var(--expense)' }}>₹{totalMonthlyCost.toLocaleString('en-IN', { maximumFractionDigits: 0 })}</div>
            </CardBody>
          </Card>
          <Card>
            <CardBody>
              <div style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 4 }}>Active Tracked Subscriptions</div>
              <div style={{ fontSize: 22, fontWeight: 700, color: 'var(--brand)' }}>{subs.filter(s => s.active).length}</div>
            </CardBody>
          </Card>
        </div>

        {loading ? (
          <div>Loading subscriptions...</div>
        ) : subs.length === 0 ? (
          <Card>
            <CardBody style={{ textAlign: 'center', padding: '32px 16px' }}>
              <RefreshCw size={32} style={{ color: 'var(--brand)', marginBottom: 8 }} />
              <div style={{ fontWeight: 600, fontSize: 15, marginBottom: 4 }}>No Subscriptions Added</div>
              <div style={{ fontSize: 13, color: 'var(--text-2)' }}>Add your monthly recurring bills (Netflix, Rent, WiFi, SIPs) to track upcoming renewals.</div>
            </CardBody>
          </Card>
        ) : (
          <Card>
            <CardBody>
              {subs.map(sub => (
                <ListRow
                  key={sub.id}
                  leading={
                    <div style={{ width: 36, height: 36, borderRadius: 8, background: 'var(--brand-soft)', color: 'var(--brand)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                      <Calendar size={18} />
                    </div>
                  }
                  trailing={
                    <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                      <div style={{ textAlign: 'right' }}>
                        <div style={{ fontWeight: 600, fontSize: 14 }}>₹{sub.amount.toLocaleString('en-IN')}</div>
                        <div style={{ fontSize: 11, color: 'var(--text-muted)' }}>due {sub.next_due_date}</div>
                      </div>
                      <Button variant="ghost" size="sm" onClick={() => handleDelete(sub.id)}>
                        <Trash2 size={15} style={{ color: 'var(--expense)' }} />
                      </Button>
                    </div>
                  }
                >
                  <ListRowText primary={sub.name} />
                  <div style={{ display: 'flex', gap: 6, marginTop: 4 }}>
                    <Chip>{sub.billing_cycle}</Chip>
                    {sub.auto_detected && <Chip>Auto-Detected</Chip>}
                  </div>
                </ListRow>
              ))}
            </CardBody>
          </Card>
        )}

        {showAdd && (
          <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20 }}>
            <div style={{ maxWidth: 400, width: '100%' }}>
              <Card>
                <CardBody>
                  <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 12 }}>Add Subscription / Bill</h3>
                  <Field value={name} onChange={e => setName(e.target.value)} placeholder="Name (e.g. Netflix, Rent)" />
                  <div style={{ marginTop: 8 }}>
                    <Field value={amount} onChange={e => setAmount(e.target.value)} placeholder="Amount (INR)" type="number" />
                  </div>
                  <div style={{ marginTop: 8 }}>
                    <select
                      value={cycle}
                      onChange={e => setCycle(e.target.value)}
                      style={{ width: '100%', padding: '10px 12px', background: 'var(--surface-2)', color: 'var(--text)', border: '1px solid var(--hairline)', borderRadius: 8, fontSize: 13 }}
                    >
                      <option value="monthly">Monthly</option>
                      <option value="yearly">Yearly</option>
                      <option value="weekly">Weekly</option>
                    </select>
                  </div>
                  <div style={{ marginTop: 8 }}>
                    <Field value={dueDate} onChange={e => setDueDate(e.target.value)} placeholder="Next Due Date" type="date" />
                  </div>
                  <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 16 }}>
                    <Button variant="secondary" onClick={() => setShowAdd(false)}>Cancel</Button>
                    <Button variant="primary" onClick={handleCreate}><CheckCircle2 size={14} /> Save Bill</Button>
                  </div>
                </CardBody>
              </Card>
            </div>
          </div>
        )}
      </Screen>
    </div>
  )
}
