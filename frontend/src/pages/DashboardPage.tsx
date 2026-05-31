import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../api/client'
import { FileUpload } from '../components/FileUpload'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'
import { useAuth } from '../store/auth'

interface DashboardStats {
  total_spent: number
  total_earned: number
  net: number
  monthly: { month: string; spent: number; earned: number }[]
  top_debits: { description: string; total: number }[]
}

const fmt = (n: number) =>
  `₹${n.toLocaleString('en-IN', { maximumFractionDigits: 2 })}`

export function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null)
  const logout = useAuth(s => s.logout)

  const fetchStats = () =>
    api.get<DashboardStats>('/txns/dashboard').then(r => setStats(r.data))

  useEffect(() => { fetchStats() }, [])

  return (
    <div style={{ maxWidth: 900, margin: '0 auto', padding: 24 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <h2 style={{ margin: 0 }}>Khata</h2>
        <nav style={{ display: 'flex', gap: 16, alignItems: 'center' }}>
          <Link to="/transactions">Transactions</Link>
          <Link to="/chat">Ask Claude</Link>
          <button onClick={logout} style={{ cursor: 'pointer' }}>Logout</button>
        </nav>
      </div>

      <FileUpload onSuccess={fetchStats} />

      {stats ? (
        <>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3,1fr)', gap: 16, marginBottom: 24 }}>
            <StatCard label="Total Spent"  value={fmt(stats.total_spent)}  color="#fee2e2" />
            <StatCard label="Total Earned" value={fmt(stats.total_earned)} color="#dcfce7" />
            <StatCard label="Net"          value={fmt(stats.net)}
              color="#eff6ff"
              valueStyle={{ color: stats.net >= 0 ? '#16a34a' : '#dc2626' }} />
          </div>

          <h3>Monthly Spend vs Earn</h3>
          <SpendEarnChart data={stats.monthly} />

          {stats.top_debits.length > 0 && (
            <>
              <h3 style={{ marginTop: 24 }}>Top Spending</h3>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 14 }}>
                <tbody>
                  {stats.top_debits.map(d => (
                    <tr key={d.description}>
                      <td style={{ padding: '5px 0', borderBottom: '1px solid #f0f0f0' }}>{d.description}</td>
                      <td style={{ textAlign: 'right', borderBottom: '1px solid #f0f0f0' }}>{fmt(d.total)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </>
      ) : (
        <p style={{ color: '#999' }}>Upload a bank statement to get started.</p>
      )}
    </div>
  )
}

function StatCard({ label, value, color, valueStyle }: {
  label: string; value: string; color: string; valueStyle?: React.CSSProperties
}) {
  return (
    <div style={{ padding: 16, background: color, borderRadius: 8 }}>
      <div style={{ fontSize: 12, color: '#666', marginBottom: 4 }}>{label}</div>
      <div style={{ fontSize: 22, fontWeight: 700, ...valueStyle }}>{value}</div>
    </div>
  )
}
