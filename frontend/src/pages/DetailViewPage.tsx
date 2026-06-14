import { useEffect, useState } from 'react'
import { useSearchParams, useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { Screen, Card, CardHeader, CardBody, Amount, Chip, EmptyState, Button } from '../components/shared'
import { ArrowLeft, ExternalLink } from 'lucide-react'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'

export function DetailViewPage() {
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const category = params.get('category')
  const month = params.get('month')
  const payee = params.get('payee')

  const label = category ? `Category: ${category}` : month ? `Month: ${month}` : payee ? `Payee: ${payee}` : 'Detail'
  const filterParam = category ? `category=${encodeURIComponent(category)}` : month ? `month=${month}` : payee ? `payee=${encodeURIComponent(payee)}` : ''

  const [data, setData] = useState<any>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!filterParam) return
    setLoading(true)
    api.get(`/txns/analytics/detail?${filterParam}`)
      .then(r => setData(r.data))
      .catch(() => setData(null))
      .finally(() => setLoading(false))
  }, [filterParam])

  return (
    <Screen title={label} actions={
      <Button variant="ghost" size="sm" onClick={() => navigate(-1)}><ArrowLeft size={14} /> Back</Button>
    }>
      {loading ? (
        <div style={{ textAlign: 'center', padding: 48, color: 'var(--text-2)' }}>Loading…</div>
      ) : !data ? (
        <EmptyState icon="📄" title="No data" description="No transactions match this filter" />
      ) : (
        <>
          {/* Totals */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 12, marginBottom: 16 }}>
            <Card><CardBody>
              <div style={{ fontSize: 10, color: 'var(--text-muted)', marginBottom: 4 }}>Total spent</div>
              <Amount paise={data.total_spent} size="lg" />
            </CardBody></Card>
            <Card><CardBody>
              <div style={{ fontSize: 10, color: 'var(--text-muted)', marginBottom: 4 }}>Total earned</div>
              <Amount paise={data.total_earned} size="lg" />
            </CardBody></Card>
            <Card><CardBody>
              <div style={{ fontSize: 10, color: 'var(--text-muted)', marginBottom: 4 }}>Transactions</div>
              <div style={{ fontSize: 22, fontWeight: 700, color: 'var(--text)', fontVariantNumeric: 'tabular-nums' }}>{data.txn_count}</div>
            </CardBody></Card>
          </div>

          {/* Trend */}
          {data.trend?.length > 0 && (
            <Card style={{ marginBottom: 16 }}>
              <CardHeader title="TREND" />
              <CardBody><SpendEarnChart data={data.trend} /></CardBody>
            </Card>
          )}

          {/* Top payees */}
          {data.top_payees?.length > 0 && (
            <Card style={{ marginBottom: 16 }}>
              <CardHeader title="TOP PAYEES" />
              <CardBody>
                {data.top_payees.map((p: any, i: number) => (
                  <div key={p.description} onClick={() => navigate(`/analytics/detail?payee=${encodeURIComponent(p.description)}`)} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 0', borderBottom: i < data.top_payees.length - 1 ? '1px solid var(--hairline)' : 'none', cursor: 'pointer' }}>
                    <span style={{ fontSize: 13, color: 'var(--text)' }}>{p.description}</span>
                    <Amount paise={p.total} size="sm" />
                  </div>
                ))}
              </CardBody>
            </Card>
          )}

          {/* View all transactions */}
          <Button variant="secondary" onClick={() => navigate(`/transactions?${filterParam}`)} style={{ width: '100%' }}>
            <ExternalLink size={14} /> View all matching transactions
          </Button>
        </>
      )}
    </Screen>
  )
}
