import { useCallback, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ArrowUpRight, ArrowDownRight, Eye, EyeOff, Plus } from 'lucide-react'
import { api } from '../api/client'
import { FileUpload } from '../components/FileUpload'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'
import { CategoryChart } from '../components/charts/CategoryChart'
import { usePrivacy } from '../store/privacy'
import { maskDescription } from '../utils/pii'
import { Screen, Card, CardHeader, CardBody, Amount, ListRow, ListRowText, Button } from '../components/shared'
import { Skeleton } from '../components/Skeleton'
import { ErrorState } from '../components/States'
import { formatDate, formatINR } from '../utils/format'
import { useVisibleRefresh } from '../hooks/useVisibleRefresh'

interface DashboardStats {
  total_spent: number; total_earned: number; total_invested: number; net: number
  monthly: { month: string; spent: number; earned: number }[]
  top_debits: { description: string; total: number }[]
}

interface AnalysisStats {
  category_breakdown: { category: string; amount: number; txn_count: number; pct: number }[]
  savings_rate_pct: number
  avg_daily_spend: number
  month_comparison: { this_month: number; last_month: number; change_pct: number }
  largest_expense: { description: string; amount: number; value_date: string } | null
  total_transactions: number
  total_invested: number
}

/**
 * The statement: the books for the whole period, set like a ledger page.
 * Single rule closes the lines; the double rule marks the final figure.
 */
function Statement({ stats, analysis }: { stats: DashboardStats; analysis: AnalysisStats | null }) {
  const savingsRate = analysis?.savings_rate_pct
  const invested = analysis?.total_invested ?? stats.total_invested
  return (
    <section aria-labelledby="statement-heading" className="statement">
      <h2 id="statement-heading" className="statement-heading">All time</h2>
      <div className="ledger">
        <div className="ledger-line">
          <span>Earned</span>
          <Amount paise={stats.total_earned} size="lg" tone="income" />
        </div>
        <div className="ledger-line">
          <span>Spent</span>
          <Amount paise={-stats.total_spent} size="lg" tone="expense" />
        </div>
        <div className="ledger-line ledger-subtotal ledger-total statement-net">
          <span>{stats.net >= 0 ? 'Saved' : 'Overspent'}</span>
          <Amount paise={stats.net} size="xl" tone={stats.net >= 0 ? 'neutral' : 'expense'} />
        </div>
      </div>
      <p className="statement-note">
        {savingsRate !== undefined && (
          <span style={{ color: savingsRate >= 20 ? 'var(--text)' : 'var(--warn)' }}>
            You kept {savingsRate.toFixed(1)}% of what you earned.
          </span>
        )}
        {invested > 0 && <span> {formatINR(invested)} went to investments and isn't counted as spend.</span>}
        <span> Transfers between your own accounts are left out.</span>
      </p>
    </section>
  )
}

function ThisMonth({ analysis }: { analysis: AnalysisStats }) {
  const { this_month, last_month, change_pct } = analysis.month_comparison
  const up = change_pct > 0
  return (
    <section className="month-panel" aria-label="This month">
      <div>
        <div className="month-panel-label">Spent this month</div>
        <Amount paise={this_month} size="lg" />
        {last_month > 0 && (
          <div className="month-panel-delta" style={{ color: up ? 'var(--expense)' : 'var(--income)' }}>
            {up ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />}
            {Math.abs(change_pct).toFixed(1)}% {up ? 'more' : 'less'} than last month ({formatINR(last_month)})
          </div>
        )}
      </div>
      {last_month > 0 && (
        <div className="month-compare" aria-hidden>
          {[['This month', this_month], ['Last month', last_month]].map(([label, v]) => (
            <div key={label as string} className="month-compare-row">
              <span>{label}</span>
              <span className="month-compare-track">
                <span className="month-compare-fill" style={{
                  display: 'block',
                  width: `${Math.min(100, (Number(v) / Math.max(this_month, last_month)) * 100)}%`,
                  background: label === 'This month' ? 'var(--text-2)' : 'var(--rule)',
                }} />
              </span>
            </div>
          ))}
        </div>
      )}
      <div className="month-panel-grid">
        <div>
          <div className="month-panel-label">Average a day</div>
          <Amount paise={analysis.avg_daily_spend} size="md" />
        </div>
        <div>
          <div className="month-panel-label">Transactions</div>
          <span className="figure" style={{ fontSize: 15 }}>{analysis.total_transactions.toLocaleString('en-IN')}</span>
        </div>
      </div>
    </section>
  )
}

export function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null)
  const [analysis, setAnalysis] = useState<AnalysisStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const navigate = useNavigate()

  const fetchAll = useCallback(() => {
    return Promise.all([
      api.get<DashboardStats>('/txns/dashboard'),
      api.get<AnalysisStats>('/txns/analysis').catch(() => ({ data: null })),
    ])
      .then(([statsRes, analysisRes]) => {
        setStats(statsRes.data)
        setAnalysis(analysisRes.data)
        setError(null)
      })
      .catch(() => setError("Couldn't load your dashboard. Check your connection and retry."))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => { void fetchAll() }, [fetchAll])
  useVisibleRefresh(fetchAll)

  const { blurMode, toggleBlur } = usePrivacy()
  const hasData = stats && (stats.total_spent > 0 || stats.total_earned > 0)

  const actions = (
    <>
      <Button variant="ghost" size="sm" onClick={toggleBlur} aria-pressed={blurMode}>
        {blurMode ? <EyeOff size={16} /> : <Eye size={16} />}
        {blurMode ? 'Show payees' : 'Hide payees'}
      </Button>
      {hasData && (
        <Button size="sm" onClick={() => navigate('/upload')}>
          <Plus size={16} /> Add statement
        </Button>
      )}
    </>
  )

  if (loading) {
    return (
      <Screen title="Dashboard" actions={actions}>
        <div className="dash-top">
          <Skeleton lines={4} height="card" />
          <Skeleton lines={3} height="card" />
        </div>
        <div className="grid grid-2" style={{ marginTop: 16 }}>
          <Skeleton lines={5} height="card" />
          <Skeleton lines={5} height="card" />
        </div>
      </Screen>
    )
  }

  if (error && !stats) {
    return (
      <Screen title="Dashboard" actions={actions}>
        <ErrorState message={error} onRetry={() => fetchAll()} />
      </Screen>
    )
  }

  if (!hasData) {
    return (
      <Screen title="Dashboard" actions={actions}>
        <div className="empty-state" style={{ marginBottom: 24 }}>
          <div className="empty-state-icon" aria-hidden>₹</div>
          <div className="empty-state-title">Your khata is empty</div>
          <div className="empty-state-msg">
            Drop a bank or credit-card statement below. Khata reads the transactions, sorts them into
            categories, and keeps card bill payments out of your spending.
          </div>
        </div>
        <FileUpload onSuccess={() => fetchAll()} />
      </Screen>
    )
  }

  return (
    <Screen title="Dashboard" actions={actions}>
      {error && <p role="status" style={{ color: 'var(--warn)', marginBottom: 12 }}>Dashboard updates delayed. Retrying automatically…</p>}
      <div className="dash-top">
        <Statement stats={stats!} analysis={analysis} />
        {analysis && <ThisMonth analysis={analysis} />}
      </div>

      <div className="grid grid-2" style={{ marginBottom: 16 }}>
        <Card>
          <CardHeader title="Earned and spent by month" />
          <CardBody>
            <SpendEarnChart data={stats!.monthly} />
          </CardBody>
        </Card>
        <Card>
          <CardHeader title="Where it went" subtitle="Select a category to see its transactions" />
          <CardBody>
            {analysis && analysis.category_breakdown.length > 0
              ? <CategoryChart
                  data={analysis.category_breakdown}
                  onCategoryClick={cat => navigate(`/transactions?category=${encodeURIComponent(cat)}`)}
                />
              : <p style={{ color: 'var(--text-2)', padding: '24px 0', textAlign: 'center' }}>
                  No categorised spending yet.
                </p>}
          </CardBody>
        </Card>
      </div>

      <div className="grid grid-2" style={{ alignItems: 'start' }}>
        <Card>
          <CardHeader title="Paid the most" />
          <div style={{ paddingTop: 8 }}>
            {stats!.top_debits.slice(0, 7).map(d => (
              <ListRow key={d.description} trailing={<Amount paise={d.total} size="sm" />}>
                <ListRowText primary={maskDescription(d.description, blurMode)} />
              </ListRow>
            ))}
          </div>
        </Card>
        {analysis?.largest_expense && (
          <Card>
            <CardHeader title="Largest single expense" />
            <CardBody>
              <Amount paise={analysis.largest_expense.amount} size="lg" />
              <div style={{ marginTop: 12 }}>
                <ListRowText
                  primary={maskDescription(analysis.largest_expense.description, blurMode)}
                  secondary={formatDate(analysis.largest_expense.value_date)}
                />
              </div>
            </CardBody>
          </Card>
        )}
      </div>
    </Screen>
  )
}
