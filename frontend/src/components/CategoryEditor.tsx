import { useEffect, useRef, useState } from 'react'
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
  const [open, setOpen] = useState(false)
  const [category, setCategory] = useState(current)
  const [scope, setScope] = useState<'single' | 'same_description' | 'contains'>('single')
  const [keyword, setKeyword] = useState('')
  const [saving, setSaving] = useState(false)
  const [updated, setUpdated] = useState<number | null>(null)
  const [isNew, setIsNew] = useState(false)
  const popRef = useRef<HTMLDivElement>(null)

  // Auto-suggest keyword from description (first meaningful token)
  useEffect(() => {
    if (scope === 'contains') {
      const first = description.split(/[/\-\s]+/).find(w => w.length > 3) ?? ''
      setKeyword(first.toUpperCase())
    }
  }, [scope, description])

  // Close on outside click
  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (popRef.current && !popRef.current.contains(e.target as Node)) setOpen(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
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
    <div style={{ position: 'relative', display: 'inline-block' }} ref={popRef}>
      <button
        onClick={() => { setOpen(o => !o); setCategory(current); setScope('single'); setUpdated(null) }}
        style={{
          background: '#f3f4f6', border: 'none', borderRadius: 4,
          padding: '2px 8px', fontSize: 11, color: '#374151',
          cursor: 'pointer', whiteSpace: 'nowrap',
        }}
        title="Click to change category"
      >
        {current} ✏
      </button>

      {open && (
        <div style={{
          position: 'absolute', zIndex: 100, top: '110%', left: 0,
          background: '#fff', border: '1px solid #e5e7eb', borderRadius: 8,
          boxShadow: '0 4px 16px rgba(0,0,0,0.12)', padding: 14, minWidth: 280,
        }}>
          <div style={{ fontSize: 12, fontWeight: 600, color: '#374151', marginBottom: 8 }}>Change Category</div>

          {/* Category input */}
          {isNew ? (
            <input
              autoFocus
              placeholder="New category name…"
              value={category}
              onChange={e => setCategory(e.target.value)}
              style={{ width: '100%', padding: '5px 8px', borderRadius: 5, border: '1px solid #d1d5db', fontSize: 13, boxSizing: 'border-box', marginBottom: 6 }}
            />
          ) : (
            <select
              value={category}
              onChange={e => setCategory(e.target.value)}
              style={{ width: '100%', padding: '5px 8px', borderRadius: 5, border: '1px solid #d1d5db', fontSize: 13, marginBottom: 6 }}
            >
              {categoryOptions.map(c => <option key={c} value={c}>{c}</option>)}
            </select>
          )}

          <button
            onClick={() => { setIsNew(n => !n); setCategory('') }}
            style={{ fontSize: 11, color: '#6366f1', background: 'none', border: 'none', cursor: 'pointer', padding: 0, marginBottom: 10 }}
          >
            {isNew ? '← Choose existing' : '+ Create new category'}
          </button>

          {/* Scope */}
          <div style={{ fontSize: 12, fontWeight: 600, color: '#374151', marginBottom: 6 }}>Apply to</div>
          {(['single', 'same_description', 'contains'] as const).map(s => (
            <label key={s} style={{ display: 'flex', alignItems: 'flex-start', gap: 6, marginBottom: 5, cursor: 'pointer', fontSize: 12 }}>
              <input type="radio" name="scope" value={s} checked={scope === s} onChange={() => setScope(s)} style={{ marginTop: 2 }} />
              <span style={{ color: '#374151' }}>{SCOPE_LABELS[s]}</span>
            </label>
          ))}

          {scope === 'contains' && (
            <input
              value={keyword}
              onChange={e => setKeyword(e.target.value)}
              placeholder="keyword to match…"
              style={{ width: '100%', padding: '4px 8px', borderRadius: 5, border: '1px solid #d1d5db', fontSize: 12, marginTop: 4, boxSizing: 'border-box' }}
            />
          )}

          {scope !== 'single' && (
            <p style={{ fontSize: 11, color: '#9ca3af', margin: '6px 0 0' }}>
              {scope === 'same_description'
                ? `Will update all: "${description.slice(0, 40)}${description.length > 40 ? '…' : ''}"`
                : keyword ? `Will update all descriptions containing "${keyword}"` : ''}
            </p>
          )}

          <div style={{ display: 'flex', gap: 8, marginTop: 10 }}>
            <button
              onClick={save}
              disabled={saving || !category.trim()}
              style={{ flex: 1, padding: '6px 0', background: '#6366f1', color: '#fff', border: 'none', borderRadius: 5, fontSize: 12, cursor: 'pointer' }}
            >
              {saving ? 'Saving…' : updated !== null ? `✓ Updated ${updated}` : 'Save'}
            </button>
            <button
              onClick={() => setOpen(false)}
              style={{ padding: '6px 10px', background: '#f3f4f6', border: 'none', borderRadius: 5, fontSize: 12, cursor: 'pointer' }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
