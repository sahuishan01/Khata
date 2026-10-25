import { useCallback, useEffect, useRef, useState, Fragment } from 'react'
import { useSearchParams } from 'react-router-dom'
import { ArrowDown, ArrowUp, ArrowUpDown, X, Calendar, Repeat, FileText } from 'lucide-react'
import { api } from '../api/client'
import { CategoryEditor } from '../components/CategoryEditor'
import { TransactionEditor, type EditableTransaction } from '../components/TransactionEditor'
import { formatINR, formatDate } from '../utils/format'
import { useVisibleRefresh } from '../hooks/useVisibleRefresh'

type TxnRow = EditableTransaction
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




export function TransactionsPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [list, setList]             = useState<TxnList | null>(null)
  const [categories, setCategories] = useState<string[]>([])
  const loadedPage = useRef(1)
  const busy = useRef(false)
  const sentinel = useRef<HTMLDivElement>(null)
  const [loadingMore, setLoadingMore] = useState(false)
  const [moreError, setMoreError] = useState<string | null>(null)
  const [sortBy, setSortBy]         = useState<SortBy>('date')
  const [sortDir, setSortDir]       = useState<SortDir>('desc')
  const [datePreset, setDatePreset] = useState<DatePreset>('all')
  const [customFrom, setCustomFrom] = useState('')
  const [customTo, setCustomTo]     = useState('')
  const [editingNotes, setEditingNotes] = useState<string | null>(null)
  const [noteText, setNoteText]     = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [refreshError, setRefreshError] = useState<string | null>(null)
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null)
  const requestId = useRef(0)
  const [duplicatesOnly, setDuplicatesOnly] = useState(false)
  const [undoDuplicateId, setUndoDuplicateId] = useState<string | null>(null)
  const [duplicateBusy, setDuplicateBusy] = useState(false)
  const [editingTransaction, setEditingTransaction] = useState<TxnRow | null>(null)

  const categoryFilter = searchParams.get('category') ?? ''

  const fetchPage = useCallback(async (p: number, append = false, checkOnly = false) => {
    if ((append || checkOnly) && busy.current) return
    const request = ++requestId.current
    busy.current = true
    if (append) { setLoadingMore(true); setMoreError(null) }
    const params = new URLSearchParams({ per_page: '50', sort_by: sortBy, sort_dir: sortDir })
    if (duplicatesOnly) params.set('duplicates_only', 'true')
    if (categoryFilter) params.set('category', categoryFilter)
    if (searchQuery) params.set('search', searchQuery)
    const { from, to } = getDateRange(datePreset, customFrom, customTo)
    if (from) params.set('from', from)
    if (to) params.set('to', to)
    try {
      let rows: TxnRow[] = []
      let response: TxnList | null = null
      let last = checkOnly ? 1 : p
      for (let current = append ? p : 1; current <= last; current++) {
        params.set('page', String(current))
        response = (await api.get<TxnList>(`/txns?${params}`)).data
        if (request !== requestId.current) return
        rows = rows.concat(response.data)
        if (checkOnly && current === 1) {
          // Polling an unchanged first page keeps the loaded history and scroll position.
          const previous = listRef.current
          if (previous && previous.total === response.total &&
              JSON.stringify(previous.data.slice(0, response.per_page)) === JSON.stringify(response.data)) {
            setRefreshError(null); setLastUpdated(new Date()); return
          }
          last = loadedPage.current
        }
        if (response.data.length === 0 || current * response.per_page >= response.total) break
      }
      if (!response) return
      const result = response
      setList(previous => ({ ...result, data: [...new Map(
        (append ? [...(previous?.data ?? []), ...rows] : rows).map(t => [t.id, t] as const)
      ).values()] }))
      loadedPage.current = response.page
      setRefreshError(null); setMoreError(null); setLastUpdated(new Date())
    } catch {
      if (request === requestId.current) {
        if (append) setMoreError('Could not load more transactions. Retry.')
        else setRefreshError('Could not refresh transactions. Retrying automatically…')
      }
    } finally {
      if (request === requestId.current) { busy.current = false; setLoadingMore(false) }
    }
  }, [sortBy, sortDir, categoryFilter, datePreset, customFrom, customTo, searchQuery, duplicatesOnly])

  const listRef = useRef<TxnList | null>(null)
  listRef.current = list
  useVisibleRefresh(() => fetchPage(loadedPage.current, false, true))
  const hasMore = !!list && list.page * list.per_page < list.total
  const loadMore = useCallback(() => {
    if (!busy.current && hasMore) void fetchPage(loadedPage.current + 1, true)
  }, [fetchPage, hasMore])
  useEffect(() => {
    const node = sentinel.current
    if (!node || !hasMore || moreError || loadingMore || !('IntersectionObserver' in window)) return
    const observer = new IntersectionObserver(entries => {
      if (entries.some(entry => entry.isIntersecting)) loadMore()
    }, { rootMargin: '400px' })
    observer.observe(node)
    return () => observer.disconnect()
  }, [loadMore, hasMore, moreError, loadingMore, list?.page])

  useEffect(() => {
    Promise.all([
      api.get<string[]>('/txns/categories'),
      api.get<{ name: string }[]>('/categories').catch(() => ({ data: [] as { name: string }[] })),
    ]).then(([txnRes, catRes]) => {
      const txnCats = txnRes.data
      const managed = catRes.data || []
      const managedNames = managed.map(c => c.name)
      setCategories([...new Set([...txnCats, ...managedNames])].sort())
    }).catch(() => {})
  }, [])

  useEffect(() => {
    loadedPage.current = 1
    setList(null); setMoreError(null)
    void fetchPage(1)
  }, [fetchPage])

  const handleSort = (col: SortBy) => {
    if (col === sortBy) {
      const newDir: SortDir = sortDir === 'desc' ? 'asc' : 'desc'
      setSortDir(newDir)
    } else {
      setSortBy(col)
      setSortDir(col === 'amount' ? 'desc' : col === 'date' ? 'desc' : 'asc')
    }
  }

  const handleCategoryUpdate = (txnId: string, newCat: string) => {
    setList(l => l ? { ...l, data: l.data.map(t => t.id === txnId ? { ...t, category: newCat } : t) } : l)
    api.get<string[]>('/txns/categories').then(r => setCategories(r.data))
    void fetchPage(loadedPage.current)
  }

  const toggleTransfer = async (id: string, val: boolean) => {
    try {
      await api.patch(`/txns/${id}/transfer`, { is_transfer: val })
      setList(l => l ? { ...l, data: l.data.map(t => t.id === id ? { ...t, is_transfer: val } : t) } : l)
      void fetchPage(loadedPage.current)
    } catch { setRefreshError('Could not save the transaction change. Please retry.') }
  }

  const saveNotes = async (id: string) => {
    try {
      await api.patch(`/txns/${id}/notes`, { notes: noteText })
      setList(l => l ? { ...l, data: l.data.map(t => t.id === id ? { ...t, notes: noteText } : t) } : l)
      setEditingNotes(null)
      void fetchPage(loadedPage.current)
    } catch { setRefreshError('Could not save the transaction change. Please retry.') }
  }

  const excludeDuplicate = async (t: TxnRow) => {
    if (!window.confirm(`Exclude this duplicate entry for ${t.description}, ${formatINR(t.amount)}, ${formatDate(t.value_date)}? Only confirm if it is the same payment as the matching record. You can undo this.`)) return
    setDuplicateBusy(true)
    try {
      await api.post(`/txns/${t.id}/duplicate`)
      setUndoDuplicateId(t.id)
      await fetchPage(loadedPage.current)
    } catch { setRefreshError('Could not exclude the duplicate. Refresh and try again.') }
    finally { setDuplicateBusy(false) }
  }
  const undoDuplicate = async () => {
    if (!undoDuplicateId) return
    setDuplicateBusy(true)
    try {
      await api.post(`/txns/${undoDuplicateId}/restore`)
      setUndoDuplicateId(null)
      await fetchPage(loadedPage.current)
    } catch { setRefreshError('Could not restore the transaction. Please retry.') }
    finally { setDuplicateBusy(false) }
  }
  const duplicateAction = (t: TxnRow) => t.possible_duplicate && (
    <button className="btn btn-secondary btn-sm" disabled={duplicateBusy}
      onClick={e => { e.stopPropagation(); void excludeDuplicate(t) }}>Exclude duplicate…</button>
  )

  const editTransactionAction = (t: TxnRow) => (
    <button className="btn btn-secondary" style={{ minHeight: 44, marginBottom: 8 }}
      onClick={event => { event.stopPropagation(); setEditingTransaction(t) }}>Edit transaction</button>
  )


  return (
    <div>
      {editingTransaction && <TransactionEditor key={editingTransaction.id} transaction={editingTransaction} categories={categories}
        onClose={() => setEditingTransaction(null)} onSaved={saved => {
          setList(previous => previous ? { ...previous, data: previous.data.map(t => t.id === saved.id ? saved : t) } : previous)
          setCategories(previous => [...new Set([...previous, saved.category])].sort())
          void fetchPage(loadedPage.current)
        }} />}
      <label style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 12 }}>
        <input type="checkbox" checked={duplicatesOnly} onChange={e => { setDuplicatesOnly(e.target.checked); loadedPage.current = 1 }} />
        Possible duplicates
      </label>
      {duplicatesOnly && <p role="status" className="text-muted">Suggestions include matching records and card payment confirmations linked to a CRED debit. Repeated payments may be legitimate. Expand a row to exclude only a confirmed duplicate.</p>}
      {undoDuplicateId && <p role="status">Duplicate excluded. <button className="btn btn-secondary btn-sm" disabled={duplicateBusy} onClick={() => void undoDuplicate()}>Undo</button></p>}
      {/* Header */}
      <div className="flex items-center justify-between mb-4" style={{ flexWrap: 'wrap', gap: 10 }}>
        <div className="flex items-center gap-3" style={{ flexWrap: 'wrap' }}>
          <h1 className="page-title">Transactions</h1>
          {list && <span className="badge badge-gray">{list.total.toLocaleString()} total</span>}
          <button className="btn btn-secondary btn-sm" onClick={() => fetchPage(loadedPage.current)}>Refresh</button>
          {lastUpdated && <span style={{ color: 'var(--text-2)', fontSize: 11 }}>Updated {lastUpdated.toLocaleTimeString()} · refreshes every 15s</span>}
          {refreshError && <span role="status" style={{ color: 'var(--warn)', fontSize: 12 }}>{refreshError}</span>}
          {categoryFilter && (
            <span className="badge badge-purple" style={{ gap: 6 }}>
              {categoryFilter}
              <button
                onClick={() => { setSearchParams({}); loadedPage.current = 1 }}
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
            onChange={e => { setSearchQuery(e.target.value); loadedPage.current = 1 }}
          />
          <select
            className="form-input"
            style={{ width: 'auto', padding: '6px 10px', fontSize: 13 }}
            value={categoryFilter}
            onChange={e => {
              const val = e.target.value
                        if (val) setSearchParams({ category: val })
              else { setSearchParams({}); loadedPage.current = 1 }
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
              setSortBy(by); setSortDir(dir); loadedPage.current = 1
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
                onClick={() => { setDatePreset(key); loadedPage.current = 1 }}
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
              onChange={e => { setCustomFrom(e.target.value); loadedPage.current = 1 }}
            />
            <span style={{ color: 'var(--text-2)', fontSize: 13 }}>to</span>
            <input
              type="date"
              className="form-input"
              style={{ width: 'auto', padding: '5px 10px', fontSize: 13 }}
              value={customTo}
              onChange={e => { setCustomTo(e.target.value); loadedPage.current = 1 }}
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
                          color: sortBy === col ? 'var(--brand)' : undefined,
                        }}
                      >
                        {label}
                        {' '}{sortBy === col
                          ? (sortDir === 'desc' ? '↓' : '↑')
                          : <span style={{ opacity: 0.3 }}>↕</span>}
                      </th>
                    ))}
                    <th style={{ width: 80 }}></th>
                  </tr>
                </thead>
                <tbody>
                  {list.data.map(t => (<Fragment key={t.id}>
                    <tr className="txn-row" onClick={() => setExpandedId(expandedId === t.id ? null : t.id)}>
                      <td style={{ whiteSpace: 'nowrap', color: 'var(--text-2)', fontSize: 12 }}>
                        {formatDate(t.value_date)}
                      </td>
                      <td className="txn-desc" style={{ color: 'var(--text)' }}>
                        <span style={{ display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>{t.description}</span>
                        {t.possible_duplicate && <span className="badge badge-gray" style={{ marginLeft: 6 }}>Possible duplicate</span>}
                        {t.notes && <span style={{ fontSize: 11, color: 'var(--text-2)', marginLeft: 6 }}>📝</span>}
                      </td>
                      <td style={{
                        textAlign: 'right', fontVariantNumeric: 'tabular-nums',
                        fontWeight: 600, whiteSpace: 'nowrap',
                        color: t.direction === 'debit' ? 'var(--expense)' : 'var(--income)',
                      }}>
                        {formatINR(t.direction === 'debit' ? -t.amount : t.amount, { sign: true })}
                      </td>
                      <td>
                        <CategoryEditor
                          txnId={t.id} current={t.category} description={t.description}
                          allCategories={categories} onUpdated={handleCategoryUpdate}
                        />
                      </td>
                      <td className="txn-actions">
                        <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                          <button className="btn btn-ghost btn-sm" style={{ fontSize: 10, padding: '2px 4px' }} onClick={e => { e.stopPropagation(); setEditingNotes(editingNotes === t.id ? null : t.id); setNoteText(t.notes || '') }} title="Notes">
                            <FileText size={12} />
                          </button>
                          <button className="btn btn-ghost btn-sm" style={{ fontSize: 10, padding: '2px 4px', color: t.is_transfer ? 'var(--warn)' : 'var(--text-2)' }} onClick={e => { e.stopPropagation(); toggleTransfer(t.id, !t.is_transfer) }} title="Toggle transfer">
                            <Repeat size={12} />
                          </button>
                        </div>
                        {editingNotes === t.id && (
                          <div style={{ marginTop: 4 }} onClick={e => e.stopPropagation()}>
                            <input className="form-input" style={{ fontSize: 11, padding: '3px 6px' }} value={noteText} onChange={e => setNoteText(e.target.value)} placeholder="Add notes…" />
                            <button className="btn btn-primary btn-sm" style={{ fontSize: 10, marginTop: 2 }} onClick={() => saveNotes(t.id)}>Save</button>
                          </div>
                        )}
                      </td>
                    </tr>
                    {expandedId === t.id && (
                      <tr className="txn-expanded">
                        <td colSpan={5} style={{ padding: '10px 12px', background: 'var(--surface-2)' }}>
                          {duplicateAction(t)}
                          {editTransactionAction(t)}
                          <div style={{ whiteSpace: 'pre-wrap', overflowWrap: 'anywhere', marginBottom: 12 }}>{t.description}</div>
                          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px 16px', fontSize: 12 }}>
                            <div><span style={{ color: 'var(--text-muted)' }}>Amount: </span><span style={{ fontWeight: 600, color: t.direction === 'debit' ? 'var(--expense)' : 'var(--income)' }}>{formatINR(t.direction === 'debit' ? -t.amount : t.amount, { sign: true })}</span></div>
                    <div><span style={{ color: 'var(--text-muted)' }}>Direction: </span><span>{t.direction}</span></div>
                            <div><span style={{ color: 'var(--text-muted)' }}>Category: </span><span>{t.category}</span></div>
                            <div><span style={{ color: 'var(--text-muted)' }}>Transaction date: </span><span>{formatDate(t.txn_date)}</span></div>
                            <div><span style={{ color: 'var(--text-muted)' }}>Value date: </span><span>{formatDate(t.value_date)}</span></div>
                            {t.balance != null && <div><span style={{ color: 'var(--text-muted)' }}>Balance: </span><span>{formatINR(t.balance)}</span></div>}
                            <div><span style={{ color: 'var(--text-muted)' }}>Bank: </span><span>{t.bank}</span></div>
                            {t.bank_ref && <div><span style={{ color: 'var(--text-muted)' }}>Reference: </span><span>{t.bank_ref}</span></div>}
                            {t.notes && <div style={{ gridColumn: '1 / -1' }}><span style={{ color: 'var(--text-muted)' }}>Notes: </span><span>{t.notes}</span></div>}
                          </div>
                        </td>
                      </tr>
                    )}
                    </Fragment>))}
                </tbody>
              </table>
            </div>
          </div>

          {/* Mobile card list */}
          <div className="panel" style={{ padding: 0, overflow: 'hidden' }} id="txn-cards">
            {list.data.map(t => (
              <div key={t.id} onClick={() => setExpandedId(expandedId === t.id ? null : t.id)} style={{ cursor: 'pointer' }}>
                <div style={{
                  display: 'flex', alignItems: 'flex-start', gap: 10,
                  padding: '12px 14px', borderBottom: expandedId !== t.id ? '1px solid var(--hairline)' : 'none',
                }}>
                  <div
                    className="txn-icon"
                    style={{
                      background: t.direction === 'debit' ? 'var(--expense-soft)' : 'var(--income-soft)',
                      color: t.direction === 'debit' ? 'var(--expense)' : 'var(--income)',
                      flexShrink: 0, marginTop: 1,
                    }}
                  >
                    {t.direction === 'debit' ? <ArrowDown size={15} /> : <ArrowUp size={15} />}
                  </div>

                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{
                      color: 'var(--text)', fontSize: 13, fontWeight: 500,
                      lineHeight: 1.35, marginBottom: 5,
                      display: '-webkit-box', WebkitLineClamp: 2,
                      WebkitBoxOrient: 'vertical', overflow: 'hidden',
                      wordBreak: 'break-word',
                    }}>
                      {t.description}
                    </div>
                    {t.possible_duplicate && <span className="badge badge-gray">Possible duplicate</span>}
                    <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
                      <span style={{ fontSize: 11, color: 'var(--text-2)', whiteSpace: 'nowrap' }}>
                        {formatDate(t.value_date)}
                      </span>
                      <CategoryEditor
                        txnId={t.id} current={t.category} description={t.description}
                        allCategories={categories} onUpdated={handleCategoryUpdate}
                      />
                      <span style={{
                        marginLeft: 'auto', fontWeight: 700, fontSize: 13.5,
                        whiteSpace: 'nowrap', fontVariantNumeric: 'tabular-nums',
                        color: t.direction === 'debit' ? 'var(--expense)' : 'var(--income)',
                      }}>
                        {formatINR(t.direction === 'debit' ? -t.amount : t.amount, { sign: true })}
                      </span>
                    </div>
                  </div>
                </div>
                {expandedId === t.id && (
                  <div style={{ padding: '8px 14px 12px', background: 'var(--surface-2)', borderBottom: '1px solid var(--hairline)', display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px 16px', fontSize: 12 }}>
                    {duplicateAction(t)}
                    <div style={{ gridColumn: '1 / -1' }}>{editTransactionAction(t)}</div>
                    <div style={{ gridColumn: '1 / -1', whiteSpace: 'pre-wrap', overflowWrap: 'anywhere' }}>{t.description}</div>
                    <div><span style={{ color: 'var(--text-muted)' }}>Transaction date: </span><span>{formatDate(t.txn_date)}</span></div>
                    <div><span style={{ color: 'var(--text-muted)' }}>Value date: </span><span>{formatDate(t.value_date)}</span></div>
                    {t.balance != null && <div><span style={{ color: 'var(--text-muted)' }}>Balance: </span><span>{formatINR(t.balance)}</span></div>}
                    <div><span style={{ color: 'var(--text-muted)' }}>Direction: </span><span>{t.direction}</span></div>
                    <div><span style={{ color: 'var(--text-muted)' }}>Category: </span><span>{t.category}</span></div>
                    <div><span style={{ color: 'var(--text-muted)' }}>Bank: </span><span>{t.bank}</span></div>
                    {t.bank_ref && <div><span style={{ color: 'var(--text-muted)' }}>Reference: </span><span>{t.bank_ref}</span></div>}
                    {t.notes && <div style={{ gridColumn: '1 / -1' }}><span style={{ color: 'var(--text-muted)' }}>Notes: </span><span>{t.notes}</span></div>}
                  </div>
                )}
              </div>
            ))}
          </div>

          <div ref={sentinel} className="flex items-center justify-between mt-4" aria-live="polite">
            <span className="text-muted">{list.data.length} of {list.total} transactions loaded</span>
            {hasMore && <button className="btn btn-secondary btn-sm" disabled={loadingMore}
              onClick={loadMore}>{loadingMore ? 'Loading more…' : moreError ?? 'Load more transactions'}</button>}
            {!hasMore && <span className="text-muted">All transactions loaded</span>}
          </div>
        </>
      )}

      <style>{`
        #txn-table  { display: block; }
        #txn-cards  { display: none; }
        #mobile-sort{ display: none; }
        .txn-desc { word-break: break-word; }
        .txn-actions { opacity: 0; transition: opacity 0.12s; width: 80px; }
        .txn-row:hover .txn-actions { opacity: 1; }
        .txn-row, .txn-row td { cursor: pointer; }
        .txn-row:hover .txn-actions button { min-height: 28px; }
        @media (max-width: 640px) {
          #txn-table  { display: none; }
          #txn-cards  { display: block; }
          #mobile-sort{ display: flex; }
        }
      `}</style>
    </div>
  )
}
