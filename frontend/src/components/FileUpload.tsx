import { useRef, useState } from 'react'
import { api } from '../api/client'

interface UploadResult {
  bank_detected: string
  rows_parsed: number
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
      const err = e as { response?: { data?: { error?: string } } }
      setError(err.response?.data?.error ?? 'Upload failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div style={{ border: '2px dashed #ccc', padding: 24, borderRadius: 8, marginBottom: 24 }}>
      <input ref={ref} type="file" accept=".csv,.xls,.xlsx" style={{ display: 'none' }}
        onChange={e => e.target.files?.[0] && upload(e.target.files[0])} />
      <button onClick={() => ref.current?.click()} disabled={loading}>
        {loading ? 'Uploading…' : '+ Upload Bank Statement (CSV / Excel)'}
      </button>
      {result && (
        <p style={{ color: 'green', marginTop: 8 }}>
          ✓ {result.bank_detected} — {result.rows_parsed} rows parsed,{' '}
          <strong>{result.inserted} added</strong>,{' '}
          {result.skipped_duplicates} duplicates skipped
        </p>
      )}
      {error && <p style={{ color: 'red', marginTop: 8 }}>✗ {error}</p>}
    </div>
  )
}
