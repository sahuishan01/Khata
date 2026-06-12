import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { Plus } from 'lucide-react'

export function AddTransactionPage() {
  const navigate = useNavigate()
  const [txnDate, setTxnDate] = useState(new Date().toISOString().slice(0, 10))
  const [valueDate, setValueDate] = useState(new Date().toISOString().slice(0, 10))
  const [description, setDescription] = useState('')
  const [amount, setAmount] = useState('')
  const [direction, setDirection] = useState<'debit' | 'credit'>('debit')
  const [category, setCategory] = useState('')
  const [bankRef, setBankRef] = useState('')
  const [notes, setNotes] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true); setError('')
    try {
      await api.post('/txns', {
        txn_date: txnDate, value_date: valueDate, description, amount: parseFloat(amount), direction, category, bank_ref: bankRef || null, notes: notes || null
      })
      navigate('/transactions')
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setError(e.response?.data?.error ?? 'Failed')
    } finally { setLoading(false) }
  }

  return (
    <div style={{ maxWidth: 500, margin: '0 auto' }}>
      <h1 className="page-title" style={{ marginBottom: 20 }}>Add Transaction</h1>
      <div className="card">
        <form onSubmit={submit}>
          <div className="form-group">
            <label className="form-label">Description</label>
            <input className="form-input" value={description} onChange={e => setDescription(e.target.value)} required placeholder="e.g. Salary, Rent payment" />
          </div>
          <div style={{ display: 'flex', gap: 10 }}>
            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">Amount (₹)</label>
              <input className="form-input" type="number" step="0.01" value={amount} onChange={e => setAmount(e.target.value)} required />
            </div>
            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">Type</label>
              <select className="form-input" value={direction} onChange={e => setDirection(e.target.value as 'debit' | 'credit')}>
                <option value="debit">Expense</option>
                <option value="credit">Income</option>
              </select>
            </div>
          </div>
          <div style={{ display: 'flex', gap: 10 }}>
            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">Transaction Date</label>
              <input className="form-input" type="date" value={txnDate} onChange={e => setTxnDate(e.target.value)} required />
            </div>
            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">Value Date</label>
              <input className="form-input" type="date" value={valueDate} onChange={e => setValueDate(e.target.value)} required />
            </div>
          </div>
          <div className="form-group">
            <label className="form-label">Category</label>
            <input className="form-input" value={category} onChange={e => setCategory(e.target.value)} placeholder="e.g. Food & Dining, Salary" />
          </div>
          <div className="form-group">
            <label className="form-label">Reference / UTR (optional)</label>
            <input className="form-input" value={bankRef} onChange={e => setBankRef(e.target.value)} />
          </div>
          <div className="form-group">
            <label className="form-label">Notes (optional)</label>
            <textarea className="form-input" rows={2} value={notes} onChange={e => setNotes(e.target.value)} />
          </div>
          {error && <p className="text-error mb-3">{error}</p>}
          <button className="btn btn-primary btn-full btn-lg" disabled={loading}><Plus size={16} /> {loading ? 'Adding…' : 'Add Transaction'}</button>
        </form>
      </div>
    </div>
  )
}
