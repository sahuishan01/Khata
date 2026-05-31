import { useRef, useState } from 'react'
import { api } from '../api/client'

interface UploadResult {
  bank_detected: string
  rows_parsed: number
  normalized: number
  inserted: number
  skipped_duplicates: number
}

export function FileUpload({ onSuccess }: { onSuccess: () => void }) {
  const [result, setResult] = useState<UploadResult | null>(null)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const ref = useRef<HTMLInputElement>(null)

  const upload = async (file: File) => {
    setLoading(true); setError(''); setResult(null)
    const fd = new FormData()
    fd.append('file', file)
    try {
      const { data } = await api.post<UploadResult>('/ingest/upload', fd)
      setResult(data)
      onSuccess()
    } catch (e: unknown) {
      const err = e as { response?: { data?: { error?: string } }; message?: string }
      setError(err.response?.data?.error ?? err.message ?? 'Upload failed')
    } finally {
      setLoading(false)
    }
  }

  const parseFailed = result && result.normalized === 0 && result.rows_parsed > 0

  return (
    <div style={{ border: '2px dashed #ccc', padding: 24, borderRadius: 8, marginBottom: 24 }}>
      <input ref={ref} type="file" accept=".csv,.xls,.xlsx" style={{ display: 'none' }}
        onChange={e => e.target.files?.[0] && upload(e.target.files[0])} />
      <button onClick={() => ref.current?.click()} disabled={loading}>
        {loading ? 'Uploading…' : '+ Upload Bank Statement (CSV / Excel)'}
      </button>

      {result && !parseFailed && (
        <p style={{ color: 'green', marginTop: 8 }}>
          ✓ <strong>{result.bank_detected}</strong> — {result.rows_parsed} rows found,{' '}
          {result.normalized} valid transactions,{' '}
          <strong>{result.inserted} new</strong>,{' '}
          {result.skipped_duplicates} already imported
        </p>
      )}

      {parseFailed && (
        <div style={{ marginTop: 8, color: '#b45309', background: '#fef3c7', padding: 12, borderRadius: 6 }}>
          <strong>⚠ {result.rows_parsed} rows found but 0 could be parsed.</strong>
          <br />
          The date or amount columns may not have been recognised.
          Check that the file is a real bank statement export (not a filtered/edited copy).
          Bank detected: <strong>{result.bank_detected}</strong>
        </div>
      )}

      {error && <p style={{ color: 'red', marginTop: 8 }}>✗ {error}</p>}
    </div>
  )
}
