import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts'

interface CategoryBucket { category: string; amount: number; pct: number }

const COLORS = [
  '#6366f1', '#f59e0b', '#10b981', '#ef4444', '#3b82f6',
  '#8b5cf6', '#f97316', '#14b8a6', '#ec4899', '#84cc16',
  '#06b6d4', '#a855f7', '#fb923c', '#94a3b8',
]

const fmt = (n: number) => `₹${n.toLocaleString('en-IN', { maximumFractionDigits: 0 })}`

export function CategoryChart({ data }: { data: CategoryBucket[] }) {
  return (
    <div style={{ display: 'flex', gap: 24, alignItems: 'center', flexWrap: 'wrap' }}>
      <ResponsiveContainer width={200} height={200}>
        <PieChart>
          <Pie data={data} dataKey="amount" cx="50%" cy="50%"
            innerRadius={55} outerRadius={90} paddingAngle={2}>
            {data.map((_, i) => (
              <Cell key={i} fill={COLORS[i % COLORS.length]} />
            ))}
          </Pie>
          <Tooltip formatter={(v) => fmt(Number(v))} />
        </PieChart>
      </ResponsiveContainer>

      <div style={{ flex: 1, minWidth: 180 }}>
        {data.map((d, i) => (
          <div key={d.category} style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
            <div style={{ width: 10, height: 10, borderRadius: 2, background: COLORS[i % COLORS.length], flexShrink: 0 }} />
            <div style={{ flex: 1, fontSize: 13 }}>{d.category}</div>
            <div style={{ fontSize: 13, fontWeight: 600, color: '#374151' }}>{fmt(d.amount)}</div>
            <div style={{ fontSize: 11, color: '#9ca3af', width: 38, textAlign: 'right' }}>
              {d.pct.toFixed(1)}%
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
