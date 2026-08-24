import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Target, Plus, Trash2, CheckCircle2 } from 'lucide-react'
import { Screen, Card, CardBody, Button, Field } from '../components/shared'

interface Goal {
  id: string
  name: string
  target_amount: number
  current_amount: number
  target_date?: string
  color_hex: string
}

export function GoalsPage() {
  const [goals, setGoals] = useState<Goal[]>([])
  const [loading, setLoading] = useState(true)
  const [showAdd, setShowAdd] = useState(false)
  const [name, setName] = useState('')
  const [target, setTarget] = useState('')
  const [current, setCurrent] = useState('')
  const [targetDate, setTargetDate] = useState('')
  const [msg, setMsg] = useState('')
  const [updateId, setUpdateId] = useState<string | null>(null)
  const [updateAmount, setUpdateAmount] = useState('')

  const fetchGoals = async () => {
    try {
      const res = await api.get<Goal[]>('/goals')
      setGoals(res.data)
    } catch (e) {
      console.error(e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchGoals()
  }, [])

  const handleCreate = async () => {
    if (!name || !target) return
    try {
      await api.post('/goals', {
        name,
        target_amount: parseFloat(target),
        current_amount: current ? parseFloat(current) : 0,
        target_date: targetDate || undefined,
      })
      setMsg('Savings Goal created!')
      setShowAdd(false)
      setName('')
      setTarget('')
      setCurrent('')
      fetchGoals()
      setTimeout(() => setMsg(''), 3000)
    } catch {
      setMsg('Failed to create goal')
    }
  }

  const handleUpdateProgress = async () => {
    if (!updateId || !updateAmount) return
    try {
      await api.patch(`/goals/${updateId}/progress`, {
        current_amount: parseFloat(updateAmount),
      })
      setUpdateId(null)
      setUpdateAmount('')
      fetchGoals()
    } catch {
      console.error('Failed to update progress')
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await api.delete(`/goals/${id}`)
      setGoals(goals.filter(g => g.id !== id))
    } catch {
      console.error('Delete failed')
    }
  }

  return (
    <div style={{ maxWidth: 650, margin: '0 auto' }}>
      <Screen
        title="Savings & Target Goals"
        subtitle="Track financial targets, emergency funds & big purchases"
        actions={
          <Button variant="primary" size="sm" onClick={() => setShowAdd(true)}>
            <Plus size={14} /> New Goal
          </Button>
        }
      >
        {msg && (
          <div style={{ background: 'var(--income-soft)', padding: '10px 14px', borderRadius: 8, fontSize: 13, marginBottom: 16 }}>
            {msg}
          </div>
        )}

        {loading ? (
          <div>Loading goals...</div>
        ) : goals.length === 0 ? (
          <Card>
            <CardBody style={{ textAlign: 'center', padding: '32px 16px' }}>
              <Target size={32} style={{ color: 'var(--brand)', marginBottom: 8 }} />
              <div style={{ fontWeight: 600, fontSize: 15, marginBottom: 4 }}>No Financial Goals Defined</div>
              <div style={{ fontSize: 13, color: 'var(--text-2)' }}>Set savings goals like Emergency Fund, Vacation, or New Car to track your target milestones.</div>
            </CardBody>
          </Card>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {goals.map(goal => {
              const pct = Math.min(100, Math.round((goal.current_amount / goal.target_amount) * 100))
              return (
                <Card key={goal.id}>
                  <CardBody>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <div style={{ width: 12, height: 12, borderRadius: '50%', background: goal.color_hex }} />
                        <span style={{ fontWeight: 600, fontSize: 15 }}>{goal.name}</span>
                      </div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span style={{ fontSize: 13, fontWeight: 700, color: 'var(--brand)' }}>{pct}%</span>
                        <Button variant="ghost" size="sm" onClick={() => handleDelete(goal.id)}>
                          <Trash2 size={15} style={{ color: 'var(--expense)' }} />
                        </Button>
                      </div>
                    </div>

                    <div style={{ background: 'var(--surface-2)', height: 8, borderRadius: 4, overflow: 'hidden', marginBottom: 12 }}>
                      <div style={{ background: goal.color_hex, width: `${pct}%`, height: '100%', borderRadius: 4, transition: 'width 0.3s ease' }} />
                    </div>

                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <div style={{ fontSize: 12, color: 'var(--text-2)' }}>
                        Saved <strong>₹{goal.current_amount.toLocaleString('en-IN')}</strong> of ₹{goal.target_amount.toLocaleString('en-IN')}
                        {goal.target_date && <span> (Target: {goal.target_date})</span>}
                      </div>
                      <Button variant="secondary" size="sm" onClick={() => { setUpdateId(goal.id); setUpdateAmount(goal.current_amount.toString()) }}>
                        Update Saved
                      </Button>
                    </div>
                  </CardBody>
                </Card>
              )
            })}
          </div>
        )}

        {showAdd && (
          <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20 }}>
            <div style={{ maxWidth: 400, width: '100%' }}>
              <Card>
                <CardBody>
                  <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 12 }}>Create Savings Goal</h3>
                  <Field value={name} onChange={e => setName(e.target.value)} placeholder="Goal Name (e.g. Emergency Fund)" />
                  <div style={{ marginTop: 8 }}>
                    <Field value={target} onChange={e => setTarget(e.target.value)} placeholder="Target Amount (INR)" type="number" />
                  </div>
                  <div style={{ marginTop: 8 }}>
                    <Field value={current} onChange={e => setCurrent(e.target.value)} placeholder="Initial Saved Amount (Optional)" type="number" />
                  </div>
                  <div style={{ marginTop: 8 }}>
                    <Field value={targetDate} onChange={e => setTargetDate(e.target.value)} placeholder="Target Date (Optional)" type="date" />
                  </div>
                  <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 16 }}>
                    <Button variant="secondary" onClick={() => setShowAdd(false)}>Cancel</Button>
                    <Button variant="primary" onClick={handleCreate}><CheckCircle2 size={14} /> Create Goal</Button>
                  </div>
                </CardBody>
              </Card>
            </div>
          </div>
        )}

        {updateId && (
          <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 100, padding: 20 }}>
            <div style={{ maxWidth: 360, width: '100%' }}>
              <Card>
                <CardBody>
                  <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 12 }}>Update Goal Saved Progress</h3>
                  <Field value={updateAmount} onChange={e => setUpdateAmount(e.target.value)} placeholder="Current Total Saved (INR)" type="number" autoFocus />
                  <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 16 }}>
                    <Button variant="secondary" onClick={() => setUpdateId(null)}>Cancel</Button>
                    <Button variant="primary" onClick={handleUpdateProgress}>Save Progress</Button>
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
