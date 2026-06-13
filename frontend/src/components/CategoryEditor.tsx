import { useEffect, useRef, useState } from 'react'
import { ChevronDown, Plus, ArrowLeft, Check } from 'lucide-react'
import { api } from '../api/client'

interface Props {
  txnId: string
  current: string
  description: string
  allCategories: string[]
  onUpdated: (txnId: string, newCategory: string) => void
}

const SCOPE_LABELS = {
  single: 'Just this transaction',
  same_description: 'All with same description',
  contains: 'All descriptions containing…',
}

export function CategoryEditor({ txnId, current, description, allCategories, onUpdated }: Props) {
  const [open, setOpen]         = useState(false)
  const [category, setCategory] = useState(current)
  const [scope, setScope]       = useState<'single' | 'same_description' | 'contains'>('single')
  const [keyword, setKeyword]   = useState('')
  const [saving, setSaving]     = useState(false)
  const [updated, setUpdated]   = useState<number | null>(null)
  const [isNew, setIsNew]       = useState(false)
  const [popPos, setPopPos]     = useState({ top: 0, left: 0 })
  const triggerRef = useRef<HTMLButtonElement>(null)
  const popRef     = useRef<HTMLDivElement>(null)

  const openEditor = () => {
    if (triggerRef.current) {
      const r = triggerRef.current.getBoundingClientRect()
      // Keep popup within right edge of viewport
      const popWidth = 280
      const left = Math.min(r.left, window.innerWidth - popWidth - 8)
      setPopPos({ top: r.bottom + 4, left })
    }
    setOpen(o => !o)
    setCategory(current)
    setScope('single')
    setUpdated(null)
    setIsNew(false)
  }

  useEffect(() => {
    if (scope === 'contains') {
      const first = description.split(/[/\-\s]+/).find(w => w.length > 3) ?? ''
      setKeyword(first.toUpperCase())
    }
  }, [scope, description])

  // Close on outside click
  useEffect(() => {
    if (!open) return
    const onDown = (e: MouseEvent) => {
      if (
        popRef.current && !popRef.current.contains(e.target as Node) &&
        triggerRef.current && !triggerRef.current.contains(e.target as Node)
      ) setOpen(false)
    }
    document.addEventListener('mousedown', onDown)
    return () => document.removeEventListener('mousedown', onDown)
  }, [open])

  // Close on scroll or resize (position would be stale)
  useEffect(() => {
    if (!open) return
    const close = () => setOpen(false)
    window.addEventListener('scroll', close, { passive: true, capture: true })
    window.addEventListener('resize', close)
    return () => {
      window.removeEventListener('scroll', close, { capture: true })
      window.removeEventListener('resize', close)
    }
  }, [open])

  const save = async () => {
    setSaving(true)
    try {
      const body: Record<string, unknown> = { category, scope }
      if (scope === 'contains') body.keyword = keyword
      const { data } = await api.put<{ updated: number }>(`/txns/${txnId}/category`, body)
      setUpdated(data.updated)
      onUpdated(txnId, category)
      setTimeout(() => { setOpen(false); setUpdated(null) }, 1200)
    } catch (e: unknown) {
      const err = e as { response?: { data?: { error?: string } } }
      alert(err.response?.data?.error ?? 'Failed to update')
    } finally {
      setSaving(false)
    }
  }

  const categoryOptions = isNew
    ? allCategories
    : [...new Set([...allCategories, category])].sort()

  return (
    <>
      <button
        ref={triggerRef}
        onClick={openEditor}
        className="badge badge-gray"
        style={{ cursor: 'pointer', gap: 4, border: 'none', background: 'var(--surface-2)' }}
        title="Change category"
      >
        {current}
        <ChevronDown size={10} style={{ opacity: 0.6 }} />
      </button>

      {open && (
        <div
          ref={popRef}
          style={{
            position: 'fixed',
            zIndex: 9999,
            top: popPos.top,
            left: popPos.left,
            width: 280,
            background: 'var(--surface)',
            border: '1px solid var(--hairline)',
            borderRadius: 'var(--r-lg)',
            boxShadow: 'var(--shadow-lg)',
            padding: 14,
          }}
        >
          <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-2)', textTransform: 'uppercase', letterSpacing: '0.07em', marginBottom: 10 }}>
            Change Category
          </div>

          {isNew ? (
            <input
              autoFocus
              className="form-input"
              placeholder="New category name…"
              value={category}
              onChange={e => setCategory(e.target.value)}
              style={{ marginBottom: 6 }}
            />
          ) : (
            <select
              className="form-input"
              value={category}
              onChange={e => setCategory(e.target.value)}
              style={{ marginBottom: 6 }}
            >
              {categoryOptions.map(c => <option key={c} value={c}>{c}</option>)}
            </select>
          )}

          <button
            onClick={() => {
              if (isNew) {
                setIsNew(false)
                setCategory(current) // restore original when going back to existing
              } else {
                setIsNew(true)
                setCategory('')
              }
            }}
            className="btn btn-ghost btn-sm"
            style={{ color: 'var(--brand)', marginBottom: 10, padding: '2px 0' }}
          >
            {isNew ? <><ArrowLeft size={12} /> Choose existing</> : <><Plus size={12} /> Create new category</>}
          </button>

          <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-2)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 7 }}>
            Apply to
          </div>

          {(['single', 'same_description', 'contains'] as const).map(s => (
            <label key={s} style={{ display: 'flex', alignItems: 'flex-start', gap: 7, marginBottom: 6, cursor: 'pointer', fontSize: 12.5 }}>
              <input
                type="radio"
                name={`scope-${txnId}`}
                value={s}
                checked={scope === s}
                onChange={() => setScope(s)}
                style={{ marginTop: 1, accentColor: 'var(--brand)' }}
              />
              <span style={{ color: 'var(--text)' }}>{SCOPE_LABELS[s]}</span>
            </label>
          ))}

          {scope === 'contains' && (
            <input
              className="form-input"
              value={keyword}
              onChange={e => setKeyword(e.target.value)}
              placeholder="keyword to match…"
              style={{ marginTop: 4 }}
            />
          )}

          {scope !== 'single' && (
            <p className="text-muted" style={{ fontSize: 11, marginTop: 5 }}>
              {scope === 'same_description'
                ? `Updates all: "${description.slice(0, 35)}${description.length > 35 ? '…' : ''}"`
                : keyword ? `Updates all containing "${keyword}"` : ''}
            </p>
          )}

          <div className="flex gap-2 mt-3">
            <button
              onClick={save}
              disabled={saving || !category.trim()}
              className="btn btn-primary btn-sm"
              style={{ flex: 1 }}
            >
              {saving ? 'Saving…' : updated !== null ? <><Check size={12} /> {updated} updated</> : 'Save'}
            </button>
            <button onClick={() => setOpen(false)} className="btn btn-secondary btn-sm">
              Cancel
            </button>
          </div>
        </div>
      )}
    </>
  )
}
