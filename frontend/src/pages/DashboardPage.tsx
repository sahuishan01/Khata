import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  TrendingDown, TrendingUp, Wallet, Percent, Calendar, Hash,
  ArrowUpRight, ArrowDownRight, AlertCircle,
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

const fmt = (n: number) => `₹${n.toLocaleString('en-IN', { maximumFractionDigits: 0 })}`
const fmtDec = (n: number) => `₹${n.toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`

export function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null)
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
        <div>
          <h1 className="page-title" style={{ marginBottom: 4 }}>Dashboard</h1>
          <p className="text-muted">Your financial overview</p>
        </div>
      </div>

      <FileUpload onSuccess={fetchAll} />

      {!hasData ? (
        <div style={{
          textAlign: 'center', padding: '64px 24px',
          background: 'var(--surface)', borderRadius: 'var(--r-xl)',
          border: '1px solid var(--border)', marginTop: 20,
        }}>
          <div style={{
            width: 64, height: 64, borderRadius: '50%',
            background: 'var(--accent-light)', color: 'var(--accent)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            fontSize: 28, margin: '0 auto 16px',
          }}>
            <Wallet size={28} />
          </div>
          <h2 style={{ fontSize: 18, marginBottom: 8 }}>No data yet</h2>
          <p style={{ color: 'var(--text-secondary)', fontSize: 14, maxWidth: 320, margin: '0 auto 20px' }}>
            Upload a bank statement to get started with your financial analysis
          </p>
        </div>
      ) : (
        <>
          <div className="grid grid-stats mb-4" style={{ marginTop: 20 }}>
            <StatCard
              label="Net Balance"
              value={fmt(stats!.net)}
              icon={<Wallet size={16} />}
              accent="accent"
              subtitle={stats!.net >= 0 ? 'Positive' : 'Negative'}
            />
            <StatCard
              label="Total Spent"
              value={fmt(stats!.total_spent)}
              icon={<TrendingDown size={16} />}
              accent="red"
              subtitle={`${analysis ? analysis.total_transactions : 0} transactions`}
            />
            <StatCard
              label="Total Earned"
              value={fmt(stats!.total_earned)}
              icon={<TrendingUp size={16} />}
              accent="green"
            />
            {analysis && (
              <StatCard
                label="Savings Rate"
                value={`${analysis.savings_rate_pct.toFixed(1)}%`}
                icon={<Percent size={16} />}
                accent={analysis.savings_rate_pct >= 20 ? 'green' : 'amber'}
                subtitle={analysis.savings_rate_pct >= 20 ? 'On track' : 'Needs improvement'}
              />
            )}
          </div>

          {analysis && analysis.month_comparison.last_month > 0 && (
            <div style={{
              background: 'var(--surface)',
              border: '1px solid var(--border)',
              borderRadius: 'var(--r-xl)',
              padding: '16px 20px',
              marginBottom: 16,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              flexWrap: 'wrap',
              gap: 8,
            }}>
              <span style={{ fontSize: 13, color: 'var(--text-secondary)' }}>
                This month spent: <strong style={{ color: 'var(--text-heading)' }}>{fmt(analysis.month_comparison.this_month)}</strong>
                <span style={{ margin: '0 8px', color: 'var(--text-muted)' }}>·</span>
                Last month: <strong style={{ color: 'var(--text-heading)' }}>{fmt(analysis.month_comparison.last_month)}</strong>
              </span>
              <span style={{
                display: 'inline-flex', alignItems: 'center', gap: 4,
                fontWeight: 600, fontSize: 13,
                color: analysis.month_comparison.change_pct > 0 ? 'var(--red)' : 'var(--green)',
              }}>
                {analysis.month_comparison.change_pct > 0
                  ? <ArrowUpRight size={14} />
                  : <ArrowDownRight size={14} />
                }
                {Math.abs(analysis.month_comparison.change_pct).toFixed(1)}% vs last month
              </span>
            </div>
          )}

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
                : <p className="text-muted" style={{ padding: '20px 0', textAlign: 'center' }}>No debit data yet.</p>}
            </div>
          </div>

          <div className="grid grid-2">
            {analysis?.largest_expense && (
              <div className="panel">
                <div className="panel-title">Largest Expense</div>
                <div style={{
                  fontSize: 28, fontWeight: 700,
                  color: 'var(--red)', marginBottom: 8,
                  letterSpacing: '-0.5px', fontVariantNumeric: 'tabular-nums',
                }}>
                  {fmtDec(analysis.largest_expense.amount)}
                </div>
                <div style={{ fontSize: 13.5, color: 'var(--text-heading)', marginBottom: 4 }}>
                  {analysis.largest_expense.description}
                </div>
                <div className="text-muted" style={{ fontSize: 12 }}>
                  <Calendar size={12} style={{ display: 'inline', marginRight: 4, verticalAlign: 'middle' }} />
                  {analysis.largest_expense.value_date}
                </div>
              </div>
            )}
            <div className="panel">
              <div className="panel-title">Top Spending</div>
              <table className="data-table">
                <tbody>
                  {stats!.top_debits.slice(0, 7).map(d => (
                    <tr key={d.description}>
                      <td className="truncate" style={{ maxWidth: 160, color: 'var(--text-heading)' }}>{d.description}</td>
                      <td className="text-right" style={{
                        color: 'var(--red)', fontVariantNumeric: 'tabular-nums',
                        fontWeight: 600, whiteSpace: 'nowrap',
                      }}>
                        {fmtDec(d.total)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {analysis && (
            <div className="grid grid-3" style={{ marginTop: 16 }}>
              <div className="panel" style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                <div style={{
                  width: 42, height: 42, borderRadius: 'var(--r-md)',
                  background: 'var(--blue-light)', color: 'var(--blue)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0,
                }}>
                  <Calendar size={18} />
                </div>
                <div>
                  <div className="stat-label">Avg Daily Spend</div>
                  <div className="stat-value" style={{ fontSize: 20 }}>{fmt(analysis.avg_daily_spend)}</div>
                </div>
              </div>
              <div className="panel" style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                <div style={{
                  width: 42, height: 42, borderRadius: 'var(--r-md)',
                  background: 'var(--accent-light)', color: 'var(--accent)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0,
                }}>
                  <Hash size={18} />
                </div>
                <div>
                  <div className="stat-label">Transactions</div>
                  <div className="stat-value" style={{ fontSize: 20 }}>{analysis.total_transactions}</div>
                </div>
              </div>
              <div className="panel" style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                <div style={{
                  width: 42, height: 42, borderRadius: 'var(--r-md)',
                  background: analysis.savings_rate_pct >= 20 ? 'var(--green-light)' : 'var(--amber-light)',
                  color: analysis.savings_rate_pct >= 20 ? 'var(--green)' : 'var(--amber)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0,
                }}>
                  <AlertCircle size={18} />
                </div>
                <div>
                  <div className="stat-label">Savings Rate</div>
                  <div className="stat-value" style={{
                    fontSize: 20,
                    color: analysis.savings_rate_pct >= 20 ? 'var(--green)' : 'var(--amber)',
                  }}>
                    {analysis.savings_rate_pct.toFixed(1)}%
                  </div>
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  )
}

function StatCard({ label, value, icon, accent, subtitle }: {
  label: string; value: string
  icon: React.ReactNode
  accent: 'red' | 'green' | 'amber' | 'accent'
  subtitle?: string
}) {
  const colors = {
    red:    { bg: 'var(--red-light)', color: 'var(--red)', bar: 'var(--red)' },
    green:  { bg: 'var(--green-light)', color: 'var(--green)', bar: 'var(--green)' },
    amber:  { bg: 'var(--amber-light)', color: 'var(--amber)', bar: 'var(--amber)' },
    accent: { bg: 'var(--accent-light)', color: 'var(--accent)', bar: 'var(--accent)' },
  }
  const c = colors[accent]
  return (
    <div className="stat-card">
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <div className="stat-icon" style={{ background: c.bg, color: c.color }}>
          {icon}
        </div>
        <div style={{ flex: 1 }}>
          <div className="stat-label">{label}</div>
          <div className="stat-value" style={{ color: c.color }}>{value}</div>
          {subtitle && <div style={{ fontSize: 11, color: 'var(--text-muted)', marginTop: 2 }}>{subtitle}</div>}
        </div>
      </div>
      <div style={{ height: 3, background: 'var(--border)', borderRadius: 2, overflow: 'hidden' }}>
        <div style={{ width: '100%', height: '100%', background: c.bar, borderRadius: 2, opacity: 0.5 }} />
      </div>
    </div>
  )
}
