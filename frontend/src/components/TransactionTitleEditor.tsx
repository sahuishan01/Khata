import { useEffect, useRef, useState, type FormEvent } from 'react'
import { api } from '../api/client'

interface Props {
  id: string
  title: string
  onSaved: (id: string, title: string) => void
  onClose: () => void
}

export function TransactionTitleEditor({ id, title, onSaved, onClose }: Props) {
  const dialog = useRef<HTMLDialogElement>(null)
  const [draft, setDraft] = useState(title)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => { dialog.current?.showModal() }, [])

  async function save(event: FormEvent) {
    event.preventDefault()
    const clean = draft.trim()
    if (!clean || clean.length > 500 || saving) return
    setSaving(true)
    setError(null)
    try {
      await api.patch(`/txns/${id}/description`, { description: clean })
      onSaved(id, clean)
      onClose()
    } catch {
      setError('Could not save the title. Check your connection and retry.')
    } finally { setSaving(false) }
  }

  return (
    <dialog ref={dialog} aria-labelledby="edit-title-heading"
      onCancel={event => { event.preventDefault(); if (!saving) onClose() }}
      style={{ width: 'min(440px, calc(100% - 32px))', boxSizing: 'border-box', background: 'var(--surface)', color: 'var(--text)', border: '1px solid var(--hairline)', borderRadius: 16, padding: 24 }}>
      <form onSubmit={save}>
        <h2 id="edit-title-heading" style={{ fontSize: 20, margin: '0 0 16px' }}>Edit title</h2>
        <label htmlFor="transaction-title" className="form-label">Title</label>
        <textarea id="transaction-title" className="form-input" autoFocus rows={3}
          value={draft} disabled={saving} maxLength={500} required
          aria-invalid={!!error} aria-describedby={error ? 'title-save-error' : undefined}
          onChange={event => { setDraft(event.target.value); setError(null) }} />
        {error && <p id="title-save-error" role="alert" style={{ color: 'var(--expense)' }}>{error}</p>}
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 16 }}>
          <button type="button" className="btn btn-secondary" style={{ minHeight: 44 }} disabled={saving} onClick={onClose}>Cancel</button>
          <button type="submit" className="btn btn-primary" style={{ minHeight: 44 }} disabled={saving || !draft.trim() || draft.trim().length > 500}>{saving ? 'Saving…' : 'Save'}</button>
        </div>
      </form>
    </dialog>
  )
}
