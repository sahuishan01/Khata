import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../api/client'
import { FileUpload } from '../components/FileUpload'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'
import { CategoryChart } from '../components/charts/CategoryChart'
import { useAuth } from '../store/auth'

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
  const logout = useAuth(s => s.logout)

  const fetchAll = () => {
    api.get<DashboardStats>('/txns/dashboard').then(r => setStats(r.data)).catch(() => {})
    api.get<AnalysisStats>('/txns/analysis').then(r => setAnalysis(r.data)).catch(() => {})
  }

  useEffect(() => { fetchAll() }, [])

  const hasData = stats && (stats.total_spent > 0 || stats.total_earned > 0)

  return (
    <div style={{ maxWidth: 960, margin: '0 auto', padding: '20px 16px', fontFamily: 'system-ui, sans-serif' }}>

      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 20 }}>
        <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700 }}>Khata</h1>
        <nav style={{ display: 'flex', gap: 16, alignItems: 'center', fontSize: 14 }}>
          <Link to="/transactions" style={{ color: '#6366f1', textDecoration: 'none' }}>Transactions</Link>
          <Link to="/chat" style={{ color: '#6366f1', textDecoration: 'none' }}>Ask Claude</Link>
          <button onClick={logout} style={{ background: 'none', border: '1px solid #e5e7eb', borderRadius: 6, padding: '4px 12px', cursor: 'pointer', fontSize: 13 }}>Logout</button>
        </nav>
      </div>

      {/* Upload */}
      <FileUpload onSuccess={fetchAll} />

      {!hasData ? (
        <p style={{ color: '#9ca3af', textAlign: 'center', marginTop: 40 }}>Upload a bank statement to see your analysis.</p>
      ) : (
        <>
          {/* Row 1: Summary cards */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', gap: 12, marginBottom: 20 }}>
            <StatCard label="Total Spent"    value={fmt(stats!.total_spent)}   color="#fee2e2" valueColor="#dc2626" />
            <StatCard label="Total Earned"   value={fmt(stats!.total_earned)}  color="#dcfce7" valueColor="#16a34a" />
            <StatCard label="Net Balance"    value={fmt(stats!.net)}
              color={stats!.net >= 0 ? '#eff6ff' : '#fff7ed'}
              valueColor={stats!.net >= 0 ? '#1d4ed8' : '#ea580c'} />
            {analysis && (
              <>
                <StatCard label="Savings Rate"
                  value={`${analysis.savings_rate_pct.toFixed(1)}%`}
                  color={analysis.savings_rate_pct >= 20 ? '#f0fdf4' : '#fefce8'}
                  valueColor={analysis.savings_rate_pct >= 20 ? '#15803d' : '#a16207'} />
                <StatCard label="Avg Daily Spend" value={fmt(analysis.avg_daily_spend)} color="#f5f3ff" valueColor="#7c3aed" />
                <StatCard label="Transactions"     value={String(analysis.total_transactions)} color="#f0f9ff" valueColor="#0369a1" />
              </>
            )}
          </div>

          {/* Row 2: Month comparison banner */}
          {analysis && analysis.month_comparison.last_month > 0 && (
            <MonthBanner mc={analysis.month_comparison} />
          )}

          {/* Row 3: Monthly chart + Category donut */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, marginBottom: 20 }}>
            <Panel title="Monthly Spend vs Earn">
              <SpendEarnChart data={stats!.monthly} />
            </Panel>
            <Panel title="Spending by Category">
              {analysis && analysis.category_breakdown.length > 0
                ? <CategoryChart data={analysis.category_breakdown} />
                : <p style={{ color: '#9ca3af', fontSize: 13 }}>No debit data yet.</p>
              }
            </Panel>
          </div>

          {/* Row 4: Largest expense + Top merchants */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
            {analysis?.largest_expense && (
              <Panel title="Largest Single Expense">
                <div style={{ fontSize: 28, fontWeight: 700, color: '#dc2626', marginBottom: 4 }}>
                  {fmtDec(analysis.largest_expense.amount)}
                </div>
                <div style={{ fontSize: 14, color: '#374151' }}>{analysis.largest_expense.description}</div>
                <div style={{ fontSize: 12, color: '#9ca3af', marginTop: 4 }}>{analysis.largest_expense.value_date}</div>
              </Panel>
            )}
            <Panel title="Top Spending">
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
                <tbody>
                  {stats!.top_debits.slice(0, 7).map(d => (
                    <tr key={d.description}>
                      <td style={{ padding: '4px 0', borderBottom: '1px solid #f3f4f6', maxWidth: 160, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{d.description}</td>
                      <td style={{ padding: '4px 0', borderBottom: '1px solid #f3f4f6', textAlign: 'right', fontVariantNumeric: 'tabular-nums', color: '#dc2626' }}>{fmtDec(d.total)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </Panel>
          </div>
        </>
      )}
    </div>
  )
}

function StatCard({ label, value, color, valueColor }: {
  label: string; value: string; color: string; valueColor?: string
}) {
  return (
    <div style={{ background: color, borderRadius: 10, padding: '12px 14px' }}>
      <div style={{ fontSize: 11, color: '#6b7280', marginBottom: 4, textTransform: 'uppercase', letterSpacing: '0.05em' }}>{label}</div>
      <div style={{ fontSize: 20, fontWeight: 700, color: valueColor ?? '#111827' }}>{value}</div>
    </div>
  )
}

function Panel({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div style={{ background: '#fff', border: '1px solid #e5e7eb', borderRadius: 10, padding: 16 }}>
      <div style={{ fontSize: 13, fontWeight: 600, color: '#374151', marginBottom: 12, textTransform: 'uppercase', letterSpacing: '0.05em' }}>{title}</div>
      {children}
    </div>
  )
}

function MonthBanner({ mc }: { mc: { this_month: number; last_month: number; change_pct: number } }) {
  const up = mc.change_pct > 0
  return (
    <div style={{
      background: up ? '#fff7ed' : '#f0fdf4',
      border: `1px solid ${up ? '#fed7aa' : '#bbf7d0'}`,
      borderRadius: 8, padding: '10px 16px', marginBottom: 16,
      display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13
    }}>
      <span style={{ color: '#374151' }}>
        This month: <strong>{fmt(mc.this_month)}</strong> &nbsp;·&nbsp; Last month: <strong>{fmt(mc.last_month)}</strong>
      </span>
      <span style={{ fontWeight: 700, color: up ? '#ea580c' : '#15803d' }}>
        {up ? '▲' : '▼'} {Math.abs(mc.change_pct).toFixed(1)}% {up ? 'more' : 'less'} than last month
      </span>
    </div>
  )
}
