import { useEffect, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { ChevronLeft, ChevronRight, ArrowDown, ArrowUp, ArrowUpDown, X, Calendar, Repeat, FileText } from 'lucide-react'
import { api } from '../api/client'
import { CategoryEditor } from '../components/CategoryEditor'

interface TxnRow {
  id: string; value_date: string; description: string
  amount: number; direction: string; bank: string; category: string
  is_transfer: boolean; notes: string
}
interface TxnList { data: TxnRow[]; total: number; page: number; per_page: number }

type SortBy    = 'date' | 'amount' | 'description' | 'category'
type SortDir   = 'asc' | 'desc'
type DatePreset = 'all' | 'week' | 'month' | 'quarter' | 'year' | 'custom'

const DATE_PRESETS: { key: DatePreset; label: string }[] = [
  { key: 'all',     label: 'All time' },
  { key: 'week',    label: 'This week' },
  { key: 'month',   label: 'This month' },
  { key: 'quarter', label: 'This quarter' },
  { key: 'year',    label: 'This year' },
  { key: 'custom',  label: 'Custom' },
]

function getDateRange(preset: DatePreset, from: string, to: string): { from?: string; to?: string } {
  const today  = new Date()
  const fmt    = (d: Date) => d.toISOString().slice(0, 10)
  switch (preset) {
    case 'week': {
      const s = new Date(today); s.setDate(today.getDate() - today.getDay())
      return { from: fmt(s), to: fmt(today) }
    }
    case 'month':
      return { from: fmt(new Date(today.getFullYear(), today.getMonth(), 1)), to: fmt(today) }
    case 'quarter': {
      const q = Math.floor(today.getMonth() / 3)
      return { from: fmt(new Date(today.getFullYear(), q * 3, 1)), to: fmt(today) }
    }
    case 'year':
      return { from: fmt(new Date(today.getFullYear(), 0, 1)), to: fmt(today) }
    case 'custom':
      return { from: from || undefined, to: to || undefined }
    default:
      return {}
  }
}

const fmt = (n: number) => `₹${n.toLocaleString('en-IN', { maximumFractionDigits: 2 })}`


export function TransactionsPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [list, setList]             = useState<TxnList | null>(null)
  const [categories, setCategories] = useState<string[]>([])
  const [page, setPage]             = useState(1)
  const [sortBy, setSortBy]         = useState<SortBy>('date')
  const [sortDir, setSortDir]       = useState<SortDir>('desc')
  const [datePreset, setDatePreset] = useState<DatePreset>('all')
  const [customFrom, setCustomFrom] = useState('')
  const [customTo, setCustomTo]     = useState('')
  const [editingNotes, setEditingNotes] = useState<string | null>(null)
  const [noteText, setNoteText]     = useState('')
  const [searchQuery, setSearchQuery] = useState('')

  const categoryFilter = searchParams.get('category') ?? ''

  const fetchPage = (
    p: number, by = sortBy, dir = sortDir,
    cat = categoryFilter, preset = datePreset, cf = customFrom, ct = customTo, search = searchQuery,
  ) => {
    const params = new URLSearchParams({ page: String(p), per_page: '50', sort_by: by, sort_dir: dir })
    if (cat) params.set('category', cat)
    if (search) params.set('search', search)
    const { from, to } = getDateRange(preset, cf, ct)
    if (from) params.set('from', from)
    if (to)   params.set('to', to)
    return api.get<TxnList>(`/txns?${params}`).then(r => setList(r.data))
  }

  useEffect(() => {
    setPage(1)
    fetchPage(1, sortBy, sortDir, categoryFilter, datePreset, customFrom, customTo, searchQuery)
    Promise.all([
      api.get<string[]>('/txns/categories'),
      api.get<any[]>('/categories').catch(() => ({ data: [] as any[] })),
    ]).then(([txnRes, catRes]) => {
      const txnCats = txnRes.data
      const managed = catRes.data || []
      const managedNames = managed.map((c: any) => c.name)
      setCategories([...new Set([...txnCats, ...managedNames])].sort())
    })
  }, [sortBy, sortDir, categoryFilter, datePreset, customFrom, customTo, searchQuery])

  useEffect(() => {
    fetchPage(page)
  }, [page])

  const handleSort = (col: SortBy) => {
    if (col === sortBy) {
      const newDir: SortDir = sortDir === 'desc' ? 'asc' : 'desc'
      setSortDir(newDir)
    } else {
      setSortBy(col)
      setSortDir(col === 'amount' ? 'desc' : col === 'date' ? 'desc' : 'asc')
    }
    setPage(1)
  }

  const handleCategoryUpdate = (txnId: string, newCat: string) => {
    setList(l => l ? { ...l, data: l.data.map(t => t.id === txnId ? { ...t, category: newCat } : t) } : l)
    api.get<string[]>('/txns/categories').then(r => setCategories(r.data))
    setTimeout(() => fetchPage(page), 1500)
  }

  const toggleTransfer = async (id: string, val: boolean) => {
    try {
      await api.patch(`/txns/${id}/transfer`, { is_transfer: val })
      setList(l => l ? { ...l, data: l.data.map(t => t.id === id ? { ...t, is_transfer: val } : t) } : l)
    } catch {}
  }

  const saveNotes = async (id: string) => {
    try {
      await api.patch(`/txns/${id}/notes`, { notes: noteText })
      setList(l => l ? { ...l, data: l.data.map(t => t.id === id ? { ...t, notes: noteText } : t) } : l)
      setEditingNotes(null)
    } catch {}
  }

  const totalPages = list ? Math.ceil(list.total / list.per_page) : 1

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-4" style={{ flexWrap: 'wrap', gap: 10 }}>
        <div className="flex items-center gap-3" style={{ flexWrap: 'wrap' }}>
          <h1 className="page-title">Transactions</h1>
          {list && <span className="badge badge-gray">{list.total.toLocaleString()} total</span>}
          {categoryFilter && (
            <span className="badge badge-purple" style={{ gap: 6 }}>
              {categoryFilter}
              <button
                onClick={() => { setSearchParams({}); setPage(1) }}
                style={{ background: 'none', border: 'none', padding: 0, cursor: 'pointer', display: 'flex', color: 'inherit', lineHeight: 1 }}
                title="Clear filter"
              >
                <X size={11} />
              </button>
            </span>
          )}
        </div>

        {/* Search + Category filter */}
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <input
            className="form-input"
            style={{ width: 180, padding: '6px 10px', fontSize: 13 }}
            placeholder="Search payee or ref…"
            value={searchQuery}
            onChange={e => { setSearchQuery(e.target.value); setPage(1) }}
          />
          <select
            className="form-input"
            style={{ width: 'auto', padding: '6px 10px', fontSize: 13 }}
            value={categoryFilter}
            onChange={e => {
              const val = e.target.value
              if (val) setSearchParams({ category: val })
              else { setSearchParams({}); setPage(1) }
            }}
          >
            <option value="">All categories</option>
            {categories.map(c => <option key={c} value={c}>{c}</option>)}
          </select>
        </div>

        {/* Mobile sort dropdown */}
        <div id="mobile-sort" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <ArrowUpDown size={14} style={{ color: 'var(--text-2)', flexShrink: 0 }} />
          <select
            className="form-input"
            style={{ width: 'auto', padding: '5px 10px', fontSize: 13 }}
            value={`${sortBy}-${sortDir}`}
            onChange={e => {
              const [by, dir] = e.target.value.split('-') as [SortBy, SortDir]
              setSortBy(by); setSortDir(dir); setPage(1)
            }}
          >
            <option value="date-desc">Date: newest first</option>
            <option value="date-asc">Date: oldest first</option>
            <option value="amount-desc">Amount: highest first</option>
            <option value="amount-asc">Amount: lowest first</option>
            <option value="category-asc">Category: A → Z</option>
            <option value="description-asc">Description: A → Z</option>
          </select>
        </div>
      </div>

      {/* Date range filter */}
      <div style={{ marginBottom: 14 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
          <Calendar size={14} style={{ color: 'var(--text-2)', flexShrink: 0 }} />
          <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
            {DATE_PRESETS.map(({ key, label }) => (
              <button
                key={key}
                onClick={() => { setDatePreset(key); setPage(1) }}
                className={datePreset === key ? 'btn btn-primary btn-sm' : 'btn btn-secondary btn-sm'}
                style={{ padding: '4px 10px', fontSize: 12 }}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
        {datePreset === 'custom' && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 8, flexWrap: 'wrap' }}>
            <input
              type="date"
              className="form-input"
              style={{ width: 'auto', padding: '5px 10px', fontSize: 13 }}
              value={customFrom}
              onChange={e => { setCustomFrom(e.target.value); setPage(1) }}
            />
            <span style={{ color: 'var(--text-2)', fontSize: 13 }}>to</span>
            <input
              type="date"
              className="form-input"
              style={{ width: 'auto', padding: '5px 10px', fontSize: 13 }}
              value={customTo}
              onChange={e => { setCustomTo(e.target.value); setPage(1) }}
            />
          </div>
        )}
      </div>

      {list && (
        <>
          {/* Desktop table */}
          <div className="panel" style={{ padding: 0 }} id="txn-table">
            <div style={{ overflowX: 'auto' }}>
              <table className="data-table" style={{ minWidth: 560 }}>
                <thead>
                  <tr>
                    {([
                      ['date',        'Date'],
                      ['description', 'Description'],
                      ['amount',      'Amount'],
                      ['category',    'Category'],
                    ] as [SortBy, string][]).map(([col, label]) => (
                      <th
                        key={col}
                        onClick={() => handleSort(col)}
                        style={{
                          cursor: 'pointer', userSelect: 'none',
                          textAlign: col === 'amount' ? 'right' : 'left',
                          whiteSpace: 'nowrap',
                          color: sortBy === col ? 'var(--accent-text)' : undefined,
                        }}
                      >
                        {label}
                        {' '}{sortBy === col
                          ? (sortDir === 'desc' ? '↓' : '↑')
                          : <span style={{ opacity: 0.3 }}>↕</span>}
                      </th>
                    ))}
                    <th style={{ whiteSpace: 'nowrap' }}>Flags</th>
                  </tr>
                </thead>
                <tbody>
                  {list.data.map(t => (
                    <tr key={t.id}>
                      <td style={{ whiteSpace: 'nowrap', color: 'var(--text-2)', fontSize: 12 }}>
                        {t.value_date}
                      </td>
                      <td className="truncate" style={{ maxWidth: 220, color: 'var(--text-heading)' }}>
                        {t.description}
                        {t.notes && <span style={{ fontSize: 11, color: 'var(--text-2)', marginLeft: 6 }}>📝</span>}
                      </td>
                      <td style={{
                        textAlign: 'right', fontVariantNumeric: 'tabular-nums',
                        fontWeight: 600, whiteSpace: 'nowrap',
                        color: t.direction === 'debit' ? 'var(--red)' : 'var(--green)',
                      }}>
                        {t.direction === 'debit' ? '−' : '+'}{fmt(t.amount)}
                      </td>
                      <td>
                        <CategoryEditor
                          txnId={t.id} current={t.category} description={t.description}
                          allCategories={categories} onUpdated={handleCategoryUpdate}
                        />
                      </td>
                      <td style={{ whiteSpace: 'nowrap' }}>
                        <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                          {t.is_transfer && <span className="badge badge-amber" style={{ fontSize: 9, cursor: 'pointer' }} onClick={() => toggleTransfer(t.id, false)}>↔</span>}
                          <button className="btn btn-ghost btn-sm" style={{ fontSize: 10, padding: '2px 4px' }} onClick={() => { setEditingNotes(editingNotes === t.id ? null : t.id); setNoteText(t.notes || '') }} title="Notes">
                            <FileText size={12} />
                          </button>
                          <button className="btn btn-ghost btn-sm" style={{ fontSize: 10, padding: '2px 4px', color: t.is_transfer ? 'var(--amber)' : 'var(--text-2)' }} onClick={() => toggleTransfer(t.id, !t.is_transfer)} title="Toggle transfer">
                            <Repeat size={12} />
                          </button>
                        </div>
                        {editingNotes === t.id && (
                          <div style={{ marginTop: 4 }}>
                            <input className="form-input" style={{ fontSize: 11, padding: '3px 6px' }} value={noteText} onChange={e => setNoteText(e.target.value)} placeholder="Add notes…" />
                            <button className="btn btn-primary btn-sm" style={{ fontSize: 10, marginTop: 2 }} onClick={() => saveNotes(t.id)}>Save</button>
                          </div>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {/* Mobile card list */}
          <div className="panel" style={{ padding: 0, overflow: 'hidden' }} id="txn-cards">
            {list.data.map(t => (
              <div key={t.id} style={{
                display: 'flex', alignItems: 'flex-start', gap: 10,
                padding: '12px 14px', borderBottom: '1px solid var(--border)',
              }}>
                <div
                  className="txn-icon"
                  style={{
                    background: t.direction === 'debit' ? 'var(--red-dim)' : 'var(--green-dim)',
                    color: t.direction === 'debit' ? 'var(--red)' : 'var(--green)',
                    flexShrink: 0, marginTop: 1,
                  }}
                >
                  {t.direction === 'debit' ? <ArrowDown size={15} /> : <ArrowUp size={15} />}
                </div>

                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{
                    color: 'var(--text-heading)', fontSize: 13, fontWeight: 500,
                    lineHeight: 1.35, marginBottom: 5,
                    display: '-webkit-box', WebkitLineClamp: 2,
                    WebkitBoxOrient: 'vertical', overflow: 'hidden',
                    wordBreak: 'break-all',
                  }}>
                    {t.description}
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
                    <span style={{ fontSize: 11, color: 'var(--text-2)', whiteSpace: 'nowrap' }}>
                      {t.value_date}
                    </span>
                    <CategoryEditor
                      txnId={t.id} current={t.category} description={t.description}
                      allCategories={categories} onUpdated={handleCategoryUpdate}
                    />
                    <span style={{
                      marginLeft: 'auto', fontWeight: 700, fontSize: 13.5,
                      whiteSpace: 'nowrap', fontVariantNumeric: 'tabular-nums',
                      color: t.direction === 'debit' ? 'var(--red)' : 'var(--green)',
                    }}>
                      {t.direction === 'debit' ? '−' : '+'}{fmt(t.amount)}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Pagination */}
          <div className="flex items-center justify-between mt-4">
            <button
              className="btn btn-secondary btn-sm"
              disabled={page === 1}
              onClick={() => setPage(p => p - 1)}
            >
              <ChevronLeft size={14} /> Prev
            </button>
            <span className="text-muted">Page {page} of {totalPages}</span>
            <button
              className="btn btn-secondary btn-sm"
              disabled={list.data.length < list.per_page}
              onClick={() => setPage(p => p + 1)}
            >
              Next <ChevronRight size={14} />
            </button>
          </div>
        </>
      )}

      <style>{`
        #txn-table  { display: block; }
        #txn-cards  { display: none; }
        #mobile-sort{ display: none; }
        @media (max-width: 640px) {
          #txn-table  { display: none; }
          #txn-cards  { display: block; }
          #mobile-sort{ display: flex; }
        }
      `}</style>
    </div>
  )
}
