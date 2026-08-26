import { useState, useRef } from 'react'
import { api } from '../api/client'
import { Upload, CheckCircle, AlertTriangle, Plus } from 'lucide-react'

export function UploadPage() {
  const [tab, setTab] = useState<'manual' | 'upload' | 'email'>('upload')
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

  const [emailConfig, setEmailConfig] = useState<{ email_address: string; imap_server: string; sync_enabled: boolean; last_synced_at?: string; last_error?: string } | null>(null)
  const [emailInput, setEmailInput] = useState('')
  const [appPasswordInput, setAppPasswordInput] = useState('')
  const [pdfPasswordInput, setPdfPasswordInput] = useState('')
  const [emailMsg, setEmailMsg] = useState('')

  const fetchEmailConfig = async () => {
    try {
      const { data } = await api.get('/ingest/email/config')
      setEmailConfig(data)
      if (data?.email_address) setEmailInput(data.email_address)
    } catch { /* ignored */ }
  }

  const saveEmailConfig = async () => {
    if (!emailInput || !appPasswordInput) return
    setLoading(true); setError('')
    try {
      await api.put('/ingest/email/config', {
        email_address: emailInput,
        app_password: appPasswordInput,
        pdf_password: pdfPasswordInput || undefined,
      })
      setEmailMsg('Gmail configuration saved securely with AES-256-GCM encryption!')
      setAppPasswordInput('')
      setPdfPasswordInput('')
      fetchEmailConfig()
      setTimeout(() => setEmailMsg(''), 4000)
    } catch (e: any) {
      setError(e.response?.data?.error ?? 'Failed to save email config')
    } finally { setLoading(false) }
  }

  const syncEmailNow = async () => {
    setLoading(true); setError(''); setResult(null)
    try {
      const { data } = await api.post('/ingest/email/sync')
      setResult({ type: 'email', message: data.message })
    } catch (e: any) {
      setError(e.response?.data?.error ?? 'Sync failed')
    } finally { setLoading(false) }
  }

  const deleteEmailConfig = async () => {
    try {
      await api.delete('/ingest/email/config')
      setEmailConfig(null)
      setEmailInput('')
      setEmailMsg('Gmail configuration disconnected.')
      setTimeout(() => setEmailMsg(''), 3000)
    } catch { setError('Failed to disconnect') }
  }

  const parseFailed = result && result.normalized === 0 && result.rows_parsed > 0

  return (
    <div style={{ maxWidth: 560, margin: '0 auto', display: 'flex', flexDirection: 'column', minHeight: 'calc(100svh - 200px)' }}>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Add Data</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Upload a statement, connect Gmail, or add a transaction manually</p>

      <div style={{ flex: 1 }}>
        {tab === 'upload' ? (
          <div className="card">
            <input ref={ref} type="file" accept=".csv,.xls,.xlsx" className="sr-only" onChange={e => e.target.files?.[0] && upload(e.target.files[0])} />
            <div className={`upload-zone${dragOver ? ' drag-over' : ''}`} onClick={() => !loading && ref.current?.click()} onDragOver={e => { e.preventDefault(); setDragOver(true) }} onDragLeave={() => setDragOver(false)} onDrop={e => { e.preventDefault(); setDragOver(false); const f = e.dataTransfer.files[0]; if (f) upload(f) }}>
              <Upload size={20} style={{ color: 'var(--brand)', margin: '0 auto 8px', display: 'block' }} />
              <p style={{ color: 'var(--text)', fontWeight: 500, fontSize: 14, marginBottom: 2 }}>{loading ? 'Uploading…' : 'Upload bank statement'}</p>
              <p className="text-muted" style={{ fontSize: 12 }}>{loading ? 'Please wait…' : 'CSV or Excel · drag & drop or click'}</p>
            </div>
            {result && !parseFailed && !result.type && (
              <div className="flex items-center gap-2 mt-3" style={{ color: 'var(--income)', fontSize: 13 }}><CheckCircle size={15} /><span><strong>{result.bank_detected}</strong> — {result.rows_parsed} rows, <strong>{result.inserted} new</strong>, {result.skipped_duplicates} duplicates</span></div>
            )}
            {parseFailed && (
              <div className="flex gap-2 mt-3" style={{ background: 'rgba(224,163,58,.1)', border: '1px solid rgba(217, 119, 6, 0.2)', borderRadius: 'var(--r-md)', padding: '10px 14px', color: 'var(--warn)', fontSize: 13 }}>
                <AlertTriangle size={15} /><span><strong>{result!.rows_parsed} rows found but 0 could be parsed.</strong> Bank detected: <strong>{result!.bank_detected}</strong></span>
              </div>
            )}
            {error && <div className="flex items-center gap-2 mt-3 text-error"><AlertTriangle size={14} />{error}</div>}
          </div>
        ) : tab === 'email' ? (
          <div className="card">
            <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>Automated Gmail Statement Sync</h3>
            <p style={{ fontSize: 13, color: 'var(--text-2)', marginBottom: 16 }}>
              Connect your Gmail using a Google App Password. Passwords are <strong>encrypted at rest (AES-256-GCM)</strong> and isolated per user via Row-Level Security.
            </p>

            {emailMsg && <p className="text-success mb-3" style={{ fontSize: 13 }}>{emailMsg}</p>}
            {error && <p className="text-error mb-3" style={{ fontSize: 13 }}>{error}</p>}
            {result?.type === 'email' && <p className="text-success mb-3" style={{ fontSize: 13 }}>{result.message}</p>}

            {emailConfig ? (
              <div style={{ background: 'var(--surface-2)', padding: 14, borderRadius: 8, marginBottom: 16 }}>
                <div style={{ fontWeight: 600, fontSize: 14 }}>Connected Email: {emailConfig.email_address}</div>
                <div style={{ fontSize: 12, color: 'var(--text-2)', marginTop: 4 }}>Server: {emailConfig.imap_server} • AES-256-GCM Encrypted</div>
                {emailConfig.last_synced_at && <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 4 }}>Last Synced: {emailConfig.last_synced_at}</div>}
                
                <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
                  <button className="btn btn-primary" onClick={syncEmailNow} disabled={loading}>Sync Email Now</button>
                  <button className="btn btn-danger" onClick={deleteEmailConfig}>Disconnect</button>
                </div>
              </div>
            ) : (
              <form onSubmit={e => { e.preventDefault(); saveEmailConfig() }}>
                <div className="form-group">
                  <label className="form-label">Gmail Address</label>
                  <input className="form-input" value={emailInput} onChange={e => setEmailInput(e.target.value)} required placeholder="yourname@gmail.com" />
                </div>
                <div className="form-group">
                  <label className="form-label">Google App Password (16-chars)</label>
                  <input className="form-input" type="password" value={appPasswordInput} onChange={e => setAppPasswordInput(e.target.value)} required placeholder="abcd efgh ijkl mnop" />
                  <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 4 }}>Generate at myaccount.google.com/apppasswords</div>
                </div>
                <div className="form-group">
                  <label className="form-label">Statement Password (Optional)</label>
                  <input className="form-input" type="password" value={pdfPasswordInput} onChange={e => setPdfPasswordInput(e.target.value)} placeholder="Password for encrypted PDF e-statements" />
                </div>
                <button className="btn btn-primary btn-full btn-lg" disabled={loading}>{loading ? 'Encrypting & Saving…' : 'Save Encrypted Config'}</button>
              </form>
            )}
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

      <div style={{ display: 'flex', gap: 0, marginTop: 20, background: 'var(--surface)', borderRadius: 'var(--r-lg)', border: '1px solid var(--hairline)', overflow: 'hidden', width: 'fit-content', marginRight: 'auto', marginLeft: 'auto' }}>
        <button onClick={() => setTab('upload')} style={{ padding: '10px 20px', border: 'none', cursor: 'pointer', fontWeight: tab === 'upload' ? 600 : 400, background: tab === 'upload' ? 'var(--brand)' : 'transparent', color: tab === 'upload' ? 'white' : 'var(--text)', transition: 'all 0.15s' }}>Upload Statement</button>
        <button onClick={() => { setTab('email'); fetchEmailConfig() }} style={{ padding: '10px 20px', border: 'none', cursor: 'pointer', fontWeight: tab === 'email' ? 600 : 400, background: tab === 'email' ? 'var(--brand)' : 'transparent', color: tab === 'email' ? 'white' : 'var(--text)', transition: 'all 0.15s' }}>Gmail Sync</button>
        <button onClick={() => setTab('manual')} style={{ padding: '10px 20px', border: 'none', cursor: 'pointer', fontWeight: tab === 'manual' ? 600 : 400, background: tab === 'manual' ? 'var(--brand)' : 'transparent', color: tab === 'manual' ? 'white' : 'var(--text)', transition: 'all 0.15s' }}>Manual Entry</button>
      </div>
    </div>
  )
}
