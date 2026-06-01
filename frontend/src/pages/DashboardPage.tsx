import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  TrendingDown, TrendingUp, Wallet, Percent, Calendar, Hash,
} from 'lucide-react'
import { api } from '../api/client'
import { FileUpload } from '../components/FileUpload'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'
import { CategoryChart } from '../components/charts/CategoryChart'

interface DashboardStats {
  total_spent: number; total_earned: number; net: number
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
}

const fmt    = (n: number) => `₹${n.toLocaleString('en-IN', { maximumFractionDigits: 0 })}`
const fmtDec = (n: number) => `₹${n.toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`

export function DashboardPage() {
  const [stats, setStats]       = useState<DashboardStats | null>(null)
  const [analysis, setAnalysis] = useState<AnalysisStats | null>(null)
  const navigate = useNavigate()

  const fetchAll = () => {
    api.get<DashboardStats>('/txns/dashboard').then(r => setStats(r.data)).catch(() => {})
    api.get<AnalysisStats>('/txns/analysis').then(r => setAnalysis(r.data)).catch(() => {})
  }

  useEffect(() => { fetchAll() }, [])

  const hasData = stats && (stats.total_spent > 0 || stats.total_earned > 0)

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="page-title">Dashboard</h1>
      </div>

      <FileUpload onSuccess={fetchAll} />

      {!hasData ? (
        <div style={{ textAlign: 'center', padding: '48px 0' }}>
          <div style={{ fontSize: 32, marginBottom: 12 }}>📊</div>
          <p style={{ color: 'var(--text-2)', fontSize: 15 }}>Upload a bank statement to see your analysis.</p>
        </div>
      ) : (
        <>
          {/* Stat cards */}
          <div className="grid grid-stats mb-4">
            <StatCard
              label="Total Spent" value={fmt(stats!.total_spent)}
              icon={<TrendingDown size={16} />}
              iconBg="var(--red-dim)" iconColor="var(--red)"
              valueColor="var(--red)"
            />
            <StatCard
              label="Total Earned" value={fmt(stats!.total_earned)}
              icon={<TrendingUp size={16} />}
              iconBg="var(--green-dim)" iconColor="var(--green)"
              valueColor="var(--green)"
            />
            <StatCard
              label="Net Balance" value={fmt(stats!.net)}
              icon={<Wallet size={16} />}
              iconBg={stats!.net >= 0 ? 'var(--blue-dim)' : 'var(--amber-dim)'}
              iconColor={stats!.net >= 0 ? 'var(--blue)' : 'var(--amber)'}
              valueColor={stats!.net >= 0 ? 'var(--blue)' : 'var(--amber)'}
            />
            {analysis && (
              <>
                <StatCard
                  label="Savings Rate"
                  value={`${analysis.savings_rate_pct.toFixed(1)}%`}
                  icon={<Percent size={16} />}
                  iconBg={analysis.savings_rate_pct >= 20 ? 'var(--green-dim)' : 'var(--amber-dim)'}
                  iconColor={analysis.savings_rate_pct >= 20 ? 'var(--green)' : 'var(--amber)'}
                  valueColor={analysis.savings_rate_pct >= 20 ? 'var(--green)' : 'var(--amber)'}
                />
                <StatCard
                  label="Avg Daily Spend" value={fmt(analysis.avg_daily_spend)}
                  icon={<Calendar size={16} />}
                  iconBg="var(--accent-dim)" iconColor="var(--accent-text)"
                />
                <StatCard
                  label="Transactions" value={String(analysis.total_transactions)}
                  icon={<Hash size={16} />}
                  iconBg="var(--blue-dim)" iconColor="var(--blue)"
                />
              </>
            )}
          </div>

          {/* Month comparison */}
          {analysis && analysis.month_comparison.last_month > 0 && (
            <MonthBanner mc={analysis.month_comparison} />
          )}

          {/* Charts row */}
          <div className="grid grid-2 mb-4">
            <div className="panel">
              <div className="panel-title">Monthly Spend vs Earn</div>
              <SpendEarnChart data={stats!.monthly} />
            </div>
            <div className="panel">
              <div className="panel-title">Spending by Category</div>
              {analysis && analysis.category_breakdown.length > 0
                ? <CategoryChart
                    data={analysis.category_breakdown}
                    onCategoryClick={cat => navigate(`/transactions?category=${encodeURIComponent(cat)}`)}
                  />
                : <p className="text-muted">No debit data yet.</p>}
            </div>
          </div>

          {/* Bottom row */}
          <div className="grid grid-2">
            {analysis?.largest_expense && (
              <div className="panel">
                <div className="panel-title">Largest Expense</div>
                <div style={{ fontSize: 26, fontWeight: 700, color: 'var(--red)', marginBottom: 6, letterSpacing: '-0.5px', fontVariantNumeric: 'tabular-nums' }}>
                  {fmtDec(analysis.largest_expense.amount)}
                </div>
                <div style={{ fontSize: 13.5, color: 'var(--text-heading)', marginBottom: 4 }}>
                  {analysis.largest_expense.description}
                </div>
                <div className="text-muted" style={{ fontSize: 12 }}>{analysis.largest_expense.value_date}</div>
              </div>
            )}
            <div className="panel">
              <div className="panel-title">Top Spending</div>
              <table className="data-table">
                <tbody>
                  {stats!.top_debits.slice(0, 7).map(d => (
                    <tr key={d.description}>
                      <td className="truncate" style={{ maxWidth: 160, color: 'var(--text-heading)' }}>{d.description}</td>
                      <td className="text-right" style={{ color: 'var(--red)', fontVariantNumeric: 'tabular-nums', fontWeight: 600, whiteSpace: 'nowrap' }}>
                        {fmtDec(d.total)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </div>
  )
}

function StatCard({ label, value, icon, iconBg, iconColor, valueColor }: {
  label: string; value: string
  icon: React.ReactNode
  iconBg: string; iconColor: string
  valueColor?: string
}) {
  return (
    <div className="stat-card">
      <div className="stat-icon" style={{ background: iconBg, color: iconColor }}>
        {icon}
      </div>
      <div>
        <div className="stat-label">{label}</div>
        <div className="stat-value" style={valueColor ? { color: valueColor } : {}}>
          {value}
        </div>
      </div>
    </div>
  )
}

function MonthBanner({ mc }: { mc: { this_month: number; last_month: number; change_pct: number } }) {
  const up = mc.change_pct > 0
  return (
    <div className={`month-banner ${up ? 'month-banner-up' : 'month-banner-down'} mb-4`}>
      <span style={{ color: 'var(--text)' }}>
        This month: <strong style={{ color: 'var(--text-heading)' }}>{fmt(mc.this_month)}</strong>
        &nbsp;·&nbsp;
        Last month: <strong style={{ color: 'var(--text-heading)' }}>{fmt(mc.last_month)}</strong>
      </span>
      <span style={{ fontWeight: 700, color: up ? 'var(--red)' : 'var(--green)' }}>
        {up ? '▲' : '▼'} {Math.abs(mc.change_pct).toFixed(1)}% {up ? 'more' : 'less'} than last month
      </span>
    </div>
  )
}
