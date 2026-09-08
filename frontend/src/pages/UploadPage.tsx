import { useState, useRef } from 'react'
import { api } from '../api/client'
import { Upload, CheckCircle, AlertTriangle, Plus } from 'lucide-react'
import { CopyButton } from '../components/shared/CopyButton'

interface EmailSyncRun {
  id: string
  started_at: string
  finished_at?: string
  status: string
  trigger: string
  full_scan: boolean
  messages_scanned: number
  attachments_seen: number
  attachments_parsed: number
  txns_imported: number
  txns_skipped: number
  errors: { stage?: string; item?: string; detail?: string }[]
  error?: string
}

const runSummary = (r: EmailSyncRun) =>
  [
    `Khata email sync run ${r.id}`,
    `status: ${r.status}  trigger: ${r.trigger}  full_scan: ${r.full_scan}`,
    `started: ${r.started_at}  finished: ${r.finished_at ?? '—'}`,
    `messages_scanned: ${r.messages_scanned}  attachments_seen: ${r.attachments_seen}  attachments_parsed: ${r.attachments_parsed}`,
    `txns_imported: ${r.txns_imported}  txns_skipped: ${r.txns_skipped}`,
    r.error ? `fatal_error: ${r.error}` : null,
    ...(r.errors ?? []).map((e, i) => `error[${i}]: [${e.stage ?? '?'}] ${e.item ?? ''} — ${e.detail ?? ''}`),
  ]
    .filter(Boolean)
    .join('\n')

export function UploadPage() {
  const [tab, setTab] = useState<'upload' | 'email' | 'manual'>('email')
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

  const [emailConfig, setEmailConfig] = useState<{ email_address: string; imap_server: string; imap_folder?: string; sync_enabled: boolean; last_synced_at?: string; last_error?: string; has_app_password?: boolean; has_pdf_password?: boolean } | null>(null)
  const [emailInput, setEmailInput] = useState('')
  const [folderInput, setFolderInput] = useState('')
  const [appPasswordInput, setAppPasswordInput] = useState('')
  const [pdfPasswordInput, setPdfPasswordInput] = useState('')
  const [emailMsg, setEmailMsg] = useState('')
  const [editingCreds, setEditingCreds] = useState(false)
  const [latestRun, setLatestRun] = useState<EmailSyncRun | null>(null)

  const fetchLatestRun = async () => {
    try {
      const { data } = await api.get('/ingest/email/runs/latest')
      setLatestRun(data)
    } catch { /* ignored */ }
  }

  const fetchEmailConfig = async () => {
    try {
      const { data } = await api.get('/ingest/email/config')
      setEmailConfig(data)
      if (data?.email_address) setEmailInput(data.email_address)
      setFolderInput(data?.imap_folder ?? '[Gmail]/All Mail')
    } catch { /* ignored */ }
    fetchLatestRun()
  }

  const hasExistingKey = !!emailConfig?.has_app_password

  const saveEmailConfig = async () => {
    if (!emailInput) return
    if (!appPasswordInput && !hasExistingKey) return
    setLoading(true); setError(''); setResult(null)
    try {
      const { data } = await api.put('/ingest/email/config', {
        email_address: emailInput,
        app_password: appPasswordInput || undefined,
        pdf_password: pdfPasswordInput || undefined,
        imap_folder: folderInput || undefined,
      })
      setAppPasswordInput('')
      setPdfPasswordInput('')
      setEditingCreds(false)
      if (data?.full_rescan_queued) {
        setEmailMsg('New key saved (AES-256-GCM). Scanning your full mailbox history for transactions…')
        await syncEmailNow()
      } else {
        setEmailMsg('Gmail configuration updated securely with AES-256-GCM encryption!')
        setTimeout(() => setEmailMsg(''), 4000)
      }
      fetchEmailConfig()
    } catch (e: any) {
      setError(e.response?.data?.error ?? 'Failed to save email config')
    } finally { setLoading(false) }
  }

  const syncEmailNow = async () => {
    setLoading(true); setError(''); setResult(null)
    try {
      const { data } = await api.post('/ingest/email/sync')
      setResult({ type: 'email', message: data.message })
      // The run is detached; give it a moment then pull the result in.
      setTimeout(fetchLatestRun, 4000)
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
      <p className="text-muted" style={{ marginBottom: 16 }}>Connect Gmail, upload a statement file, or enter transactions manually</p>

      {/* Top Segmented Tab Switcher */}
      <div style={{ display: 'flex', gap: 4, marginBottom: 20, background: 'var(--surface-2)', padding: 4, borderRadius: 'var(--r-lg)', border: '1px solid var(--hairline)' }}>
        <button onClick={() => { setTab('email'); fetchEmailConfig() }} style={{ flex: 1, padding: '9px 12px', border: 'none', borderRadius: 'var(--r-md)', cursor: 'pointer', fontWeight: tab === 'email' ? 600 : 400, background: tab === 'email' ? 'var(--brand)' : 'transparent', color: tab === 'email' ? 'white' : 'var(--text-2)', transition: 'all 0.15s' }}>Gmail Sync</button>
        <button onClick={() => setTab('upload')} style={{ flex: 1, padding: '9px 12px', border: 'none', borderRadius: 'var(--r-md)', cursor: 'pointer', fontWeight: tab === 'upload' ? 600 : 400, background: tab === 'upload' ? 'var(--brand)' : 'transparent', color: tab === 'upload' ? 'white' : 'var(--text-2)', transition: 'all 0.15s' }}>Upload Statement</button>
        <button onClick={() => setTab('manual')} style={{ flex: 1, padding: '9px 12px', border: 'none', borderRadius: 'var(--r-md)', cursor: 'pointer', fontWeight: tab === 'manual' ? 600 : 400, background: tab === 'manual' ? 'var(--brand)' : 'transparent', color: tab === 'manual' ? 'white' : 'var(--text-2)', transition: 'all 0.15s' }}>Manual Entry</button>
      </div>

      <div style={{ flex: 1 }}>
        {tab === 'upload' ? (
          <div className="card">
            <input ref={ref} type="file" accept=".csv,.xls,.xlsx,.pdf" className="sr-only" onChange={e => e.target.files?.[0] && upload(e.target.files[0])} />
            <div className={`upload-zone${dragOver ? ' drag-over' : ''}`} onClick={() => !loading && ref.current?.click()} onDragOver={e => { e.preventDefault(); setDragOver(true) }} onDragLeave={() => setDragOver(false)} onDrop={e => { e.preventDefault(); setDragOver(false); const f = e.dataTransfer.files[0]; if (f) upload(f) }}>
              <Upload size={20} style={{ color: 'var(--brand)', margin: '0 auto 8px', display: 'block' }} />
              <p style={{ color: 'var(--text)', fontWeight: 500, fontSize: 14, marginBottom: 2 }}>{loading ? 'Uploading…' : 'Upload bank statement'}</p>
              <p className="text-muted" style={{ fontSize: 12 }}>{loading ? 'Please wait…' : 'PDF, CSV or Excel · drag & drop or click'}</p>
            </div>
            {result && !parseFailed && !result.type && (
              <div className="flex items-center gap-2 mt-3" style={{ color: 'var(--income)', fontSize: 13 }}><CheckCircle size={15} /><span><strong>{result.bank_detected}</strong> — {result.rows_parsed} rows, <strong>{result.inserted} new</strong>, {result.skipped_duplicates} duplicates</span></div>
            )}
            {parseFailed && (
              <div className="flex gap-2 mt-3" style={{ background: 'rgba(224,163,58,.1)', border: '1px solid rgba(217, 119, 6, 0.2)', borderRadius: 'var(--r-md)', padding: '10px 14px', color: 'var(--warn)', fontSize: 13 }}>
                <AlertTriangle size={15} /><span><strong>{result!.rows_parsed} rows found but 0 could be parsed.</strong> Bank detected: <strong>{result!.bank_detected}</strong></span>
              </div>
            )}
            {error && (
              <div className="flex items-center gap-2 mt-3 text-error" style={{ flexWrap: 'wrap' }}>
                <AlertTriangle size={14} />{error}
                <CopyButton text={error} label="Copy" />
              </div>
            )}
          </div>
        ) : tab === 'email' ? (
          <div className="card">
            <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>Automated Gmail Statement Sync</h3>
            <p style={{ fontSize: 13, color: 'var(--text-2)', marginBottom: 16 }}>
              Connect your Gmail using a Google App Password. Passwords are <strong>encrypted at rest (AES-256-GCM)</strong> and isolated per user via Row-Level Security.
            </p>

            {emailMsg && <p className="text-success mb-3" style={{ fontSize: 13 }}>{emailMsg}</p>}
            {error && (
              <div className="flex items-center gap-2 mb-3 text-error" style={{ fontSize: 13, flexWrap: 'wrap' }}>
                <span>{error}</span>
                <CopyButton text={error} label="Copy" />
              </div>
            )}
            {result?.type === 'email' && <p className="text-success mb-3" style={{ fontSize: 13 }}>{result.message}</p>}

            {latestRun && (
              <div style={{ background: 'var(--surface-2)', padding: 14, borderRadius: 8, marginBottom: 16, fontSize: 12 }}>
                <div className="flex items-center gap-2" style={{ justifyContent: 'space-between', flexWrap: 'wrap' }}>
                  <strong style={{ fontSize: 13 }}>
                    Last sync:{' '}
                    <span style={{ color: latestRun.status === 'ok' ? 'var(--income)' : latestRun.status === 'error' ? 'var(--expense)' : 'var(--warn)' }}>
                      {latestRun.status}
                    </span>
                  </strong>
                  <div style={{ display: 'flex', gap: 6 }}>
                    <CopyButton text={runSummary(latestRun)} label="Copy details" />
                    <button className="btn btn-secondary btn-sm" onClick={fetchLatestRun}>Refresh</button>
                  </div>
                </div>
                <div style={{ color: 'var(--text-2)', marginTop: 6 }}>
                  {latestRun.messages_scanned} emails scanned · {latestRun.attachments_seen} attachments · {latestRun.txns_imported} imported · {latestRun.txns_skipped} skipped
                </div>
                {latestRun.messages_scanned === 0 && latestRun.status === 'ok' && (
                  <div style={{ color: 'var(--warn)', marginTop: 6 }}>
                    No matching emails found. Bank statement mails are often archived out of the inbox — the scan now covers all mail, so re-check your sender/subject filters if this persists.
                  </div>
                )}
                {latestRun.error && <div style={{ color: 'var(--expense)', marginTop: 6 }}>{latestRun.error}</div>}
                {latestRun.errors?.length > 0 && (
                  <ul style={{ margin: '6px 0 0', paddingLeft: 16, color: 'var(--text-2)' }}>
                    {latestRun.errors.map((e, i) => (
                      <li key={i}>[{e.stage}] {e.item}: {e.detail}</li>
                    ))}
                  </ul>
                )}
              </div>
            )}

            {emailConfig && !editingCreds ? (
              <div style={{ background: 'var(--surface-2)', padding: 14, borderRadius: 8, marginBottom: 16 }}>
                <div style={{ fontWeight: 600, fontSize: 14 }}>Connected Email: {emailConfig.email_address}</div>
                <div style={{ fontSize: 12, color: 'var(--text-2)', marginTop: 4 }}>Server: {emailConfig.imap_server} • Folder: {emailConfig.imap_folder ?? '[Gmail]/All Mail'} • AES-256-GCM Encrypted</div>
                <div className="flex items-center gap-2" style={{ fontSize: 12, color: 'var(--income)', marginTop: 6 }}>
                  <CheckCircle size={14} />
                  <span>App Password on file: <strong>•••• •••• •••• ••••</strong></span>
                </div>
                <div style={{ fontSize: 12, color: 'var(--text-2)', marginTop: 4 }}>
                  Statement password: {emailConfig.has_pdf_password ? 'set' : 'not set'}
                </div>
                {emailConfig.last_synced_at
                  ? <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 4 }}>Last Synced: {emailConfig.last_synced_at}</div>
                  : <div style={{ fontSize: 11, color: 'var(--warn)', marginTop: 4 }}>Not yet synced — a full mailbox scan will run on next sync</div>}

                <div style={{ display: 'flex', gap: 8, marginTop: 12, flexWrap: 'wrap' }}>
                  <button className="btn btn-primary" onClick={syncEmailNow} disabled={loading}>Sync Email Now</button>
                  <button className="btn btn-secondary" onClick={() => { setEditingCreds(true); setEmailInput(emailConfig.email_address); setAppPasswordInput(''); setPdfPasswordInput(''); setError(''); setEmailMsg('') }}>Update Key</button>
                  <button className="btn btn-danger" onClick={deleteEmailConfig}>Disconnect</button>
                </div>
              </div>
            ) : (
              <form onSubmit={e => { e.preventDefault(); saveEmailConfig() }}>
                {hasExistingKey && (
                  <p style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 12 }}>
                    A key is already stored. Enter a new App Password to rotate it — saving a new key triggers a full rescan of all your transactions. Leave it blank to keep the current key.
                  </p>
                )}
                <div className="form-group">
                  <label className="form-label">Gmail Address</label>
                  <input className="form-input" value={emailInput} onChange={e => setEmailInput(e.target.value)} required placeholder="yourname@gmail.com" />
                </div>
                <div className="form-group">
                  <label className="form-label">Google App Password (16-chars)</label>
                  <input className="form-input" type="password" value={appPasswordInput} onChange={e => setAppPasswordInput(e.target.value)} required={!hasExistingKey} placeholder={hasExistingKey ? 'Leave blank to keep current key' : 'abcd efgh ijkl mnop'} />
                  <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 4 }}>Generate at myaccount.google.com/apppasswords</div>
                </div>
                <div className="form-group">
                  <label className="form-label">IMAP Folder</label>
                  <input className="form-input" value={folderInput} onChange={e => setFolderInput(e.target.value)} placeholder="[Gmail]/All Mail" />
                  <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 4 }}>Default scans all mail — statement emails are usually auto-archived out of the inbox.</div>
                </div>
                <div className="form-group">
                  <label className="form-label">Statement Password (Optional)</label>
                  <input className="form-input" type="password" value={pdfPasswordInput} onChange={e => setPdfPasswordInput(e.target.value)} placeholder={emailConfig?.has_pdf_password ? 'Leave blank to keep current' : 'Password for encrypted PDF e-statements'} />
                </div>
                <button className="btn btn-primary btn-full btn-lg" disabled={loading}>{loading ? 'Encrypting & Saving…' : hasExistingKey ? 'Save New Key & Rescan' : 'Save Encrypted Config'}</button>
                {editingCreds && (
                  <button type="button" className="btn btn-secondary btn-full" style={{ marginTop: 8 }} onClick={() => { setEditingCreds(false); setAppPasswordInput(''); setPdfPasswordInput(''); setError('') }}>Cancel</button>
                )}
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
              {error && (
                <div className="flex items-center gap-2 mb-3 text-error" style={{ flexWrap: 'wrap' }}>
                  <span>{error}</span>
                  <CopyButton text={error} label="Copy" />
                </div>
              )}
              <button className="btn btn-primary btn-full btn-lg" disabled={loading}><Plus size={16} /> {loading ? 'Adding…' : 'Add Transaction'}</button>
            </form>
          </div>
        )}
      </div>
    </div>
  )
}
