import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  TrendingDown, TrendingUp, Wallet, Percent, Calendar, Hash,
  ArrowUpRight, ArrowDownRight, AlertCircle, TrendingUp as InvestIcon,
  Eye, EyeOff,
} from 'lucide-react'
import { api } from '../api/client'
import { FileUpload } from '../components/FileUpload'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'
import { CategoryChart } from '../components/charts/CategoryChart'
import { usePrivacy } from '../store/privacy'
import { maskDescription } from '../utils/pii'
import { Screen, Card, CardHeader, CardBody, StatCard, Amount, ListRow, ListRowText, Button, EmptyState } from '../components/shared'
import { formatDate } from '../utils/format'

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

export function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null)
  const [analysis, setAnalysis] = useState<AnalysisStats | null>(null)
  const navigate = useNavigate()

  const fetchAll = () => {
    api.get<DashboardStats>('/txns/dashboard').then(r => setStats(r.data)).catch(() => {})
    api.get<AnalysisStats>('/txns/analysis').then(r => setAnalysis(r.data)).catch(() => {})
  }

  useEffect(() => { fetchAll() }, [])

  const { blurMode, toggleBlur } = usePrivacy()
  const hasData = stats && (stats.total_spent > 0 || stats.total_earned > 0)

  const blurToggle = (
    <Button variant="ghost" size="sm" onClick={toggleBlur}>
      {blurMode ? <EyeOff size={14} /> : <Eye size={14} />}
      <span style={{ fontSize: 11, marginLeft: 4 }}>{blurMode ? 'Blurred' : 'Visible'}</span>
    </Button>
  )

  return (
    <Screen title="Dashboard" subtitle="Your financial overview" actions={blurToggle}>
      <FileUpload onSuccess={fetchAll} />

      {!hasData ? (
        <div style={{ marginTop: 20 }}>
          <EmptyState
            icon="💰"
            title="No data yet"
            description="Upload a bank statement to get started with your financial analysis"
          />
        </div>
      ) : (
        <>
          <div className="grid grid-stats" style={{ marginTop: 20 }}>
            <StatCard
              label="Net Balance"
              value={<Amount paise={stats!.net} size="lg" />}
              icon={<Wallet size={16} />}
              color="brand"
            />
            <StatCard
              label="Total Spent"
              value={<Amount paise={stats!.total_spent} size="lg" />}
              icon={<TrendingDown size={16} />}
              color="expense"
            />
            <StatCard
              label="Total Earned"
              value={<Amount paise={stats!.total_earned} size="lg" />}
              icon={<TrendingUp size={16} />}
              color="income"
            />
            {analysis && (
              <StatCard
                label="Savings Rate"
                value={`${analysis.savings_rate_pct.toFixed(1)}%`}
                icon={<Percent size={16} />}
                color={analysis.savings_rate_pct >= 20 ? 'income' : 'amber'}
              />
            )}
            {(analysis?.total_invested ?? 0) > 0 && (
              <StatCard
                label="Invested"
                value={<Amount paise={analysis!.total_invested} size="lg" />}
                icon={<InvestIcon size={16} />}
                color="brand"
              />
            )}
          </div>

          {analysis && analysis.month_comparison.last_month > 0 && (
            <div style={{ marginBottom: 16 }}>
              <Card>
                <CardBody style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 8 }}>
                  <span style={{ fontSize: 13, color: 'var(--text-2)' }}>
                    This month spent: <strong style={{ color: 'var(--text)' }}><Amount paise={analysis.month_comparison.this_month} size="md" /></strong>
                    <span style={{ margin: '0 8px', color: 'var(--text-muted)' }}>·</span>
                    Last month: <strong style={{ color: 'var(--text)' }}><Amount paise={analysis.month_comparison.last_month} size="md" /></strong>
                  </span>
                  <span style={{
                    display: 'inline-flex', alignItems: 'center', gap: 4,
                    fontWeight: 600, fontSize: 13,
                    color: analysis.month_comparison.change_pct > 0 ? 'var(--expense)' : 'var(--income)',
                  }}>
                    {analysis.month_comparison.change_pct > 0
                      ? <ArrowUpRight size={14} />
                      : <ArrowDownRight size={14} />
                    }
                    {Math.abs(analysis.month_comparison.change_pct).toFixed(1)}% vs last month
                  </span>
                </CardBody>
              </Card>
            </div>
          )}

          <div className="grid grid-2" style={{ marginBottom: 16 }}>
            <Card>
              <CardHeader title="Monthly Spend vs Earn" />
              <CardBody>
                <SpendEarnChart data={stats!.monthly} />
              </CardBody>
            </Card>
            <Card>
              <CardHeader title="Spending by Category" />
              <CardBody>
                {analysis && analysis.category_breakdown.length > 0
                  ? <CategoryChart
                      data={analysis.category_breakdown}
                      onCategoryClick={cat => navigate(`/transactions?category=${encodeURIComponent(cat)}`)}
                    />
                  : <p style={{ color: 'var(--text-2)', fontSize: 14, padding: '20px 0', textAlign: 'center' }}>No spending data yet. Upload a statement or add a transaction to see your category breakdown.</p>}
              </CardBody>
            </Card>
          </div>

          <div className="grid grid-2" style={{ marginBottom: 16 }}>
            {analysis?.largest_expense && (
              <Card>
                <CardHeader title="Largest Expense" />
                <CardBody>
                  <Amount paise={analysis.largest_expense.amount} size="lg" />
                  <ListRowText
                    primary={maskDescription(analysis.largest_expense.description, blurMode)}
                    secondary={formatDate(analysis.largest_expense.value_date)}
                  />
                </CardBody>
              </Card>
            )}
            <Card>
              <CardHeader title="Top Spending" />
              {stats!.top_debits.slice(0, 7).map(d => (
                <ListRow
                  key={d.description}
                  trailing={<Amount paise={d.total} size="sm" />}
                >
                  <ListRowText primary={maskDescription(d.description, blurMode)} />
                </ListRow>
              ))}
            </Card>
          </div>

          {analysis && (
            <div className="grid grid-3">
              <StatCard
                label="Avg Daily Spend"
                value={<Amount paise={analysis.avg_daily_spend} size="lg" />}
                icon={<Calendar size={16} />}
                color="brand"
              />
              <StatCard
                label="Transactions"
                value={`${analysis.total_transactions}`}
                icon={<Hash size={16} />}
                color="brand"
              />
              <StatCard
                label="Savings Rate"
                value={`${analysis.savings_rate_pct.toFixed(1)}%`}
                icon={<AlertCircle size={16} />}
                color={analysis.savings_rate_pct >= 20 ? 'income' : 'amber'}
              />
            </div>
          )}
        </>
      )}
    </Screen>
  )
}
