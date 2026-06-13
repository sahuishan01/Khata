import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts'
import { formatINR } from '../../utils/format'

interface CategoryBucket { category: string; amount: number; pct: number }

const COLORS = [
  '#6366f1', '#f59e0b', '#10b981', '#ef4444', '#3b82f6',
  '#8b5cf6', '#f97316', '#14b8a6', '#ec4899', '#84cc16',
  '#06b6d4', '#a855f7', '#fb923c', '#94a3b8',
]



interface Props {
  data: CategoryBucket[]
  onCategoryClick?: (category: string) => void
}

export function CategoryChart({ data, onCategoryClick }: Props) {
  return (
    <div style={{ display: 'flex', gap: 24, alignItems: 'center', flexWrap: 'wrap' }}>
      <ResponsiveContainer width={200} height={200}>
        <PieChart>
          <Pie
            data={data}
            dataKey="amount"
            cx="50%" cy="50%"
            innerRadius={55} outerRadius={90} paddingAngle={2}
            nameKey="category"
            onClick={onCategoryClick ? (entry) => onCategoryClick((entry as unknown as CategoryBucket).category) : undefined}
            style={onCategoryClick ? { cursor: 'pointer' } : undefined}
          >
            {data.map((_, i) => (
              <Cell key={i} fill={COLORS[i % COLORS.length]} />
            ))}
          </Pie>
          <Tooltip formatter={(v) => formatINR(Number(v))} />
        </PieChart>
      </ResponsiveContainer>

      <div style={{ flex: 1, minWidth: 160 }}>
        {data.map((d, i) => (
          <div
            key={d.category}
            onClick={() => onCategoryClick?.(d.category)}
            style={{
              display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6,
              cursor: onCategoryClick ? 'pointer' : 'default',
              borderRadius: 6, padding: '3px 4px', margin: '0 -4px 4px',
              transition: 'background 0.12s',
            }}
            onMouseEnter={e => { if (onCategoryClick) (e.currentTarget as HTMLElement).style.background = 'var(--surface-2)' }}
            onMouseLeave={e => { (e.currentTarget as HTMLElement).style.background = '' }}
          >
            <div style={{ width: 10, height: 10, borderRadius: 2, background: COLORS[i % COLORS.length], flexShrink: 0 }} />
            <div style={{ flex: 1, fontSize: 13, color: 'var(--text)' }}>{d.category}</div>
            <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text)' }}>{formatINR(d.amount)}</div>
            <div style={{ fontSize: 11, color: 'var(--text-2)', width: 38, textAlign: 'right' }}>
              {d.pct.toFixed(1)}%
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
