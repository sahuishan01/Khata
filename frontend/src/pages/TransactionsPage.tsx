import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../api/client'
import { CategoryEditor } from '../components/CategoryEditor'

interface TxnRow {
  id: string; value_date: string; description: string
  amount: number; direction: string; bank: string; category: string
}
interface TxnList { data: TxnRow[]; total: number; page: number; per_page: number }

const fmt = (n: number) => `₹${n.toLocaleString('en-IN', { maximumFractionDigits: 2 })}`

export function TransactionsPage() {
  const [list, setList] = useState<TxnList | null>(null)
  const [categories, setCategories] = useState<string[]>([])
  const [page, setPage] = useState(1)

  const fetchPage = (p: number) =>
    api.get<TxnList>(`/txns?page=${p}&per_page=50`).then(r => setList(r.data))

  useEffect(() => {
    fetchPage(page)
    api.get<string[]>('/txns/categories').then(r => setCategories(r.data))
  }, [page])

  const handleCategoryUpdate = (txnId: string, newCat: string) => {
    // Optimistically update the local list; also refresh categories in case new one was created
    setList(l => l ? {
      ...l,
      data: l.data.map(t => t.id === txnId ? { ...t, category: newCat } : t)
    } : l)
    api.get<string[]>('/txns/categories').then(r => setCategories(r.data))
    // Full refresh after 1.5s to pick up bulk updates
    setTimeout(() => fetchPage(page), 1500)
  }

  return (
    <div style={{ maxWidth: 960, margin: '0 auto', padding: 24, fontFamily: 'system-ui, sans-serif' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <h2 style={{ margin: 0, fontSize: 20 }}>Transactions</h2>
        <Link to="/" style={{ color: '#6366f1', textDecoration: 'none', fontSize: 14 }}>← Dashboard</Link>
      </div>

      {list && (
        <>
          <p style={{ color: '#6b7280', fontSize: 13, marginBottom: 8 }}>{list.total} transactions</p>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr style={{ borderBottom: '2px solid #e5e7eb', background: '#f9fafb' }}>
                <th style={{ textAlign: 'left', padding: '8px 6px', fontWeight: 600, color: '#374151' }}>Date</th>
                <th style={{ textAlign: 'left', padding: '8px 6px', fontWeight: 600, color: '#374151' }}>Description</th>
                <th style={{ textAlign: 'right', padding: '8px 6px', fontWeight: 600, color: '#374151' }}>Amount</th>
                <th style={{ textAlign: 'left', padding: '8px 6px', fontWeight: 600, color: '#374151' }}>Category</th>
              </tr>
            </thead>
            <tbody>
              {list.data.map(t => (
                <tr key={t.id} style={{ borderBottom: '1px solid #f3f4f6' }}>
                  <td style={{ padding: '7px 6px', whiteSpace: 'nowrap', color: '#9ca3af', fontSize: 12 }}>{t.value_date}</td>
                  <td style={{ padding: '7px 6px', maxWidth: 300, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', color: '#111827' }}>
                    {t.description}
                  </td>
                  <td style={{ padding: '7px 6px', textAlign: 'right', fontVariantNumeric: 'tabular-nums', fontWeight: 600,
                    color: t.direction === 'debit' ? '#dc2626' : '#16a34a' }}>
                    {t.direction === 'debit' ? '-' : '+'}{fmt(t.amount)}
                  </td>
                  <td style={{ padding: '7px 6px' }}>
                    <CategoryEditor
                      txnId={t.id}
                      current={t.category}
                      description={t.description}
                      allCategories={categories}
                      onUpdated={handleCategoryUpdate}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <div style={{ marginTop: 16, display: 'flex', gap: 8, alignItems: 'center' }}>
            <button disabled={page === 1} onClick={() => setPage(p => p - 1)}
              style={{ padding: '5px 12px', borderRadius: 5, border: '1px solid #d1d5db', cursor: 'pointer', background: page === 1 ? '#f9fafb' : '#fff' }}>
              ← Prev
            </button>
            <span style={{ fontSize: 12, color: '#6b7280' }}>Page {page} · {list.total} total</span>
            <button disabled={list.data.length < list.per_page} onClick={() => setPage(p => p + 1)}
              style={{ padding: '5px 12px', borderRadius: 5, border: '1px solid #d1d5db', cursor: 'pointer', background: list.data.length < list.per_page ? '#f9fafb' : '#fff' }}>
              Next →
            </button>
          </div>
        </>
      )}
    </div>
  )
}
