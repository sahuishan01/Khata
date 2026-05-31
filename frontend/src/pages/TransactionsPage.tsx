import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../api/client'

interface TxnRow {
  id: string; value_date: string; description: string
  amount: number; direction: string; bank: string
}
interface TxnList { data: TxnRow[]; total: number; page: number; per_page: number }

const fmt = (n: number) => `₹${n.toLocaleString('en-IN', { maximumFractionDigits: 2 })}`

export function TransactionsPage() {
  const [list, setList] = useState<TxnList | null>(null)
  const [page, setPage] = useState(1)

  useEffect(() => {
    api.get<TxnList>(`/txns?page=${page}&per_page=50`).then(r => setList(r.data))
  }, [page])

  return (
    <div style={{ maxWidth: 900, margin: '0 auto', padding: 24 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2 style={{ margin: 0 }}>Transactions</h2>
        <Link to="/">← Dashboard</Link>
      </div>
      {list && (
        <>
          <p style={{ color: '#666', fontSize: 13 }}>{list.total} total transactions</p>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 14 }}>
            <thead>
              <tr style={{ borderBottom: '2px solid #eee' }}>
                <th style={{ textAlign: 'left', padding: '8px 4px' }}>Date</th>
                <th style={{ textAlign: 'left', padding: '8px 4px' }}>Description</th>
                <th style={{ textAlign: 'right', padding: '8px 4px' }}>Amount</th>
                <th style={{ textAlign: 'left', padding: '8px 4px' }}>Bank</th>
              </tr>
            </thead>
            <tbody>
              {list.data.map(t => (
                <tr key={t.id} style={{ borderBottom: '1px solid #f5f5f5' }}>
                  <td style={{ padding: '6px 4px', whiteSpace: 'nowrap' }}>{t.value_date}</td>
                  <td style={{ padding: '6px 4px' }}>{t.description}</td>
                  <td style={{ padding: '6px 4px', textAlign: 'right',
                    color: t.direction === 'debit' ? '#dc2626' : '#16a34a', fontVariantNumeric: 'tabular-nums' }}>
                    {t.direction === 'debit' ? '-' : '+'}{fmt(t.amount)}
                  </td>
                  <td style={{ padding: '6px 4px', color: '#666' }}>{t.bank}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <div style={{ marginTop: 16, display: 'flex', gap: 8, alignItems: 'center' }}>
            <button disabled={page === 1} onClick={() => setPage(p => p - 1)}>← Prev</button>
            <span style={{ fontSize: 13 }}>Page {page}</span>
            <button disabled={list.data.length < list.per_page} onClick={() => setPage(p => p + 1)}>Next →</button>
          </div>
        </>
      )}
    </div>
  )
}
