import { useRef, useState } from 'react'
import { Upload, CheckCircle, AlertTriangle } from 'lucide-react'
import { api } from '../api/client'

interface UploadResult {
  bank_detected: string
  rows_parsed: number
  normalized: number
  inserted: number
  skipped_duplicates: number
}

export function FileUpload({ onSuccess }: { onSuccess: () => void }) {
  const [result, setResult]   = useState<UploadResult | null>(null)
  const [error, setError]     = useState('')
  const [loading, setLoading] = useState(false)
  const [dragOver, setDragOver] = useState(false)
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

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(false)
    const file = e.dataTransfer.files[0]
    if (file) upload(file)
  }

  const parseFailed = result && result.normalized === 0 && result.rows_parsed > 0

  return (
    <div className="mb-6">
      <input
        ref={ref}
        type="file"
        accept=".csv,.xls,.xlsx"
        className="sr-only"
        onChange={e => e.target.files?.[0] && upload(e.target.files[0])}
      />

      <div
        className={`upload-zone${dragOver ? ' drag-over' : ''}`}
        onClick={() => !loading && ref.current?.click()}
        onDragOver={e => { e.preventDefault(); setDragOver(true) }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
      >
        <Upload
          size={20}
          style={{ color: 'var(--accent-text)', margin: '0 auto 8px', display: 'block' }}
        />
        <p style={{ color: 'var(--text-heading)', fontWeight: 500, fontSize: 14, marginBottom: 2 }}>
          {loading ? 'Uploading…' : 'Upload bank statement'}
        </p>
        <p className="text-muted" style={{ fontSize: 12 }}>
          {loading ? 'Please wait…' : 'CSV or Excel · drag & drop or click'}
        </p>
      </div>

      {result && !parseFailed && (
        <div className="flex items-center gap-2 mt-3" style={{ color: 'var(--green)', fontSize: 13 }}>
          <CheckCircle size={15} style={{ flexShrink: 0 }} />
          <span>
            <strong>{result.bank_detected}</strong> — {result.rows_parsed} rows,{' '}
            <strong>{result.inserted} new</strong> imported,{' '}
            {result.skipped_duplicates} duplicates skipped
          </span>
        </div>
      )}

      {parseFailed && (
        <div className="flex gap-2 mt-3" style={{
          background: 'var(--amber-dim)',
          border: '1px solid rgba(217, 119, 6, 0.2)',
          borderRadius: 'var(--r-md)',
          padding: '10px 14px',
          color: 'var(--amber)',
          fontSize: 13,
        }}>
          <AlertTriangle size={15} style={{ flexShrink: 0, marginTop: 1 }} />
          <span>
            <strong>{result!.rows_parsed} rows found but 0 could be parsed.</strong>
            {' '}Date or amount columns not recognised. Check this is a real bank export.
            {' '}Bank detected: <strong>{result!.bank_detected}</strong>
          </span>
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 mt-3 text-error">
          <AlertTriangle size={14} style={{ flexShrink: 0 }} />
          {error}
        </div>
      )}
    </div>
  )
}
