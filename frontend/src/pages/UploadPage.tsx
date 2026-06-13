import { useState, useRef } from 'react'
import { api } from '../api/client'
import { Upload, CheckCircle, AlertTriangle, Plus } from 'lucide-react'

export function UploadPage() {
  const [tab, setTab] = useState<'upload' | 'manual'>('upload')
  const [result, setResult] = useState<any>(null)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const [dragOver, setDragOver] = useState(false)
  const ref = useRef<HTMLInputElement>(null)

  // Manual entry form
  const [desc, setDesc] = useState('')
  const [amount, setAmount] = useState('')
  const [direction, setDirection] = useState<'debit' | 'credit'>('debit')
  const [txnDate, setTxnDate] = useState(new Date().toISOString().slice(0, 10))
  const [valueDate, setValueDate] = useState(new Date().toISOString().slice(0, 10))
  const [category, setCategory] = useState('')
  const [notes, setNotes] = useState('')

  const upload = async (file: File) => {
    setLoading(true); setError(''); setResult(null)
    const fd = new FormData()
    fd.append('file', file)
    try {
      const { data } = await api.post('/ingest/upload', fd)
      setResult(data)
    } catch (e: unknown) {
      const err = e as { response?: { data?: { error?: string } }; message?: string }
      setError(err.response?.data?.error ?? err.message ?? 'Upload failed')
    } finally { setLoading(false) }
  }

  const addManual = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true); setError('')
    try {
      await api.post('/txns', { txn_date: txnDate, value_date: valueDate, description: desc, amount: parseFloat(amount), direction, category: category || 'Miscellaneous', notes: notes || null })
      setDesc(''); setAmount(''); setCategory(''); setNotes('')
      setResult({ type: 'manual', message: 'Transaction added' })
    } catch (err: unknown) {
      const e = err as { response?: { data?: { error?: string } } }
      setError(e.response?.data?.error ?? 'Failed')
    } finally { setLoading(false) }
  }

  const parseFailed = result && result.normalized === 0 && result.rows_parsed > 0

  return (
    <div style={{ maxWidth: 560, margin: '0 auto', display: 'flex', flexDirection: 'column', minHeight: 'calc(100svh - 200px)' }}>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Add Data</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Upload a statement or add a transaction manually</p>

      <div style={{ flex: 1 }}>
        {tab === 'upload' ? (
          <div className="card">
            <input ref={ref} type="file" accept=".csv,.xls,.xlsx" className="sr-only" onChange={e => e.target.files?.[0] && upload(e.target.files[0])} />
            <div className={`upload-zone${dragOver ? ' drag-over' : ''}`} onClick={() => !loading && ref.current?.click()} onDragOver={e => { e.preventDefault(); setDragOver(true) }} onDragLeave={() => setDragOver(false)} onDrop={e => { e.preventDefault(); setDragOver(false); const f = e.dataTransfer.files[0]; if (f) upload(f) }}>
              <Upload size={20} style={{ color: 'var(--accent-text)', margin: '0 auto 8px', display: 'block' }} />
              <p style={{ color: 'var(--text-heading)', fontWeight: 500, fontSize: 14, marginBottom: 2 }}>{loading ? 'Uploading…' : 'Upload bank statement'}</p>
              <p className="text-muted" style={{ fontSize: 12 }}>{loading ? 'Please wait…' : 'CSV or Excel · drag & drop or click'}</p>
            </div>
            {result && !parseFailed && !result.type && (
              <div className="flex items-center gap-2 mt-3" style={{ color: 'var(--green)', fontSize: 13 }}><CheckCircle size={15} /><span><strong>{result.bank_detected}</strong> — {result.rows_parsed} rows, <strong>{result.inserted} new</strong>, {result.skipped_duplicates} duplicates</span></div>
            )}
            {parseFailed && (
              <div className="flex gap-2 mt-3" style={{ background: 'var(--amber-dim)', border: '1px solid rgba(217, 119, 6, 0.2)', borderRadius: 'var(--r-md)', padding: '10px 14px', color: 'var(--amber)', fontSize: 13 }}>
                <AlertTriangle size={15} /><span><strong>{result!.rows_parsed} rows found but 0 could be parsed.</strong> Bank detected: <strong>{result!.bank_detected}</strong></span>
              </div>
            )}
            {error && <div className="flex items-center gap-2 mt-3 text-error"><AlertTriangle size={14} />{error}</div>}
          </div>
        ) : (
          <div className="card">
            <form onSubmit={addManual}>
              <div className="form-group"><label className="form-label">Description</label><input className="form-input" value={desc} onChange={e => setDesc(e.target.value)} required placeholder="e.g. Salary, Rent" /></div>
              <div style={{ display: 'flex', gap: 10 }}>
                <div className="form-group" style={{ flex: 1 }}><label className="form-label">Amount</label><input className="form-input" type="number" step="0.01" value={amount} onChange={e => setAmount(e.target.value)} required /></div>
                <div className="form-group" style={{ flex: 1 }}>
                  <label className="form-label">Type</label>
                  <select className="form-input" value={direction} onChange={e => setDirection(e.target.value as 'debit' | 'credit')}>
                    <option value="debit">Expense</option><option value="credit">Income</option>
                  </select>
                </div>
              </div>
              <div style={{ display: 'flex', gap: 10 }}>
                <div className="form-group" style={{ flex: 1 }}><label className="form-label">Date</label><input className="form-input" type="date" value={txnDate} onChange={e => setTxnDate(e.target.value)} required /></div>
                <div className="form-group" style={{ flex: 1 }}><label className="form-label">Value Date</label><input className="form-input" type="date" value={valueDate} onChange={e => setValueDate(e.target.value)} required /></div>
              </div>
              <div className="form-group"><label className="form-label">Category</label><input className="form-input" value={category} onChange={e => setCategory(e.target.value)} placeholder="e.g. Food & Dining" /></div>
              <div className="form-group"><label className="form-label">Notes</label><textarea className="form-input" rows={2} value={notes} onChange={e => setNotes(e.target.value)} /></div>
              {result?.type === 'manual' && <p className="text-success mb-3">Transaction added successfully!</p>}
              {error && <p className="text-error mb-3">{error}</p>}
              <button className="btn btn-primary btn-full btn-lg" disabled={loading}><Plus size={16} /> {loading ? 'Adding…' : 'Add Transaction'}</button>
            </form>
          </div>
        )}
      </div>

      <div style={{ display: 'flex', gap: 0, marginTop: 20, background: 'var(--surface)', borderRadius: 'var(--r-lg)', border: '1px solid var(--border)', overflow: 'hidden', width: 'fit-content', marginRight: 'auto', marginLeft: 'auto' }}>
        <button onClick={() => setTab('upload')} style={{ padding: '10px 24px', border: 'none', cursor: 'pointer', fontWeight: tab === 'upload' ? 600 : 400, background: tab === 'upload' ? 'var(--accent)' : 'transparent', color: tab === 'upload' ? 'white' : 'var(--text)', transition: 'all 0.15s' }}>Upload Statement</button>
        <button onClick={() => setTab('manual')} style={{ padding: '10px 24px', border: 'none', cursor: 'pointer', fontWeight: tab === 'manual' ? 600 : 400, background: tab === 'manual' ? 'var(--accent)' : 'transparent', color: tab === 'manual' ? 'white' : 'var(--text)', transition: 'all 0.15s' }}>Manual Entry</button>
      </div>
    </div>
  )
}
