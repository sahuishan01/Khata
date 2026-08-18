import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts'
import { formatINR } from '../../utils/format'

interface CategoryBucket { category: string; amount: number; pct: number }

const CURATED_PALETTE = [
  '#8479F2', '#2EC27E', '#EE6B4D', '#E0A33A', '#74b9ff',
  '#a29bfe', '#00F0FF', '#05FFB0', '#FF2A6D', '#F59E0B',
  '#10b981', '#3b82f6', '#8b5cf6', '#ec4899',
]

interface Props {
  data: CategoryBucket[]
  onCategoryClick?: (category: string) => void
}

function CategoryTooltip({ active, payload }: any) {
  if (active && payload && payload.length) {
    const data: CategoryBucket = payload[0].payload
    return (
      <div style={{
        background: 'var(--surface-2)',
        border: '1px solid var(--hairline)',
        borderRadius: 8,
        padding: '8px 12px',
        boxShadow: '0 8px 24px rgba(0,0,0,0.4)',
        backdropFilter: 'blur(8px)',
      }}>
        <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--text)' }}>{data.category}</div>
        <div style={{ fontSize: 13, fontWeight: 700, color: 'var(--brand)', marginTop: 2, fontVariantNumeric: 'tabular-nums' }}>
          {formatINR(data.amount)} ({data.pct.toFixed(1)}%)
        </div>
      </div>
    )
  }
  return null
}

export function CategoryChart({ data, onCategoryClick }: Props) {
  const totalAmount = data.reduce((acc, curr) => acc + curr.amount, 0)

  return (
    <div style={{ display: 'flex', gap: 20, alignItems: 'center', flexWrap: 'wrap' }}>
      <div style={{ position: 'relative', width: 190, height: 190, flexShrink: 0 }}>
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={data}
              dataKey="amount"
              cx="50%" cy="50%"
              innerRadius={58} outerRadius={86} paddingAngle={3}
              nameKey="category"
              onClick={onCategoryClick ? (entry) => onCategoryClick((entry as unknown as CategoryBucket).category) : undefined}
              style={onCategoryClick ? { cursor: 'pointer' } : undefined}
              stroke="none"
            >
              {data.map((_, i) => (
                <Cell key={i} fill={CURATED_PALETTE[i % CURATED_PALETTE.length]} />
              ))}
            </Pie>
            <Tooltip content={<CategoryTooltip />} />
          </PieChart>
        </ResponsiveContainer>

        <div style={{
          position: 'absolute', inset: 0,
          display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
          pointerEvents: 'none', textAlign: 'center', padding: 8
        }}>
          <span style={{ fontSize: 10, fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>
            Total
          </span>
          <span style={{ fontSize: 13, fontWeight: 700, color: 'var(--text)', fontVariantNumeric: 'tabular-nums', marginTop: 2 }}>
            {formatINR(totalAmount)}
          </span>
        </div>
      </div>

      <div style={{ flex: 1, minWidth: 160 }}>
        {data.map((d, i) => (
          <div
            key={d.category}
            onClick={() => onCategoryClick?.(d.category)}
            style={{
              display: 'flex', alignItems: 'center', gap: 10,
              cursor: onCategoryClick ? 'pointer' : 'default',
              borderRadius: 8, padding: '5px 8px', margin: '0 -8px 4px',
              transition: 'all 0.15s ease',
              border: '1px solid transparent'
            }}
            onMouseEnter={e => {
              const el = e.currentTarget as HTMLElement
              el.style.background = 'var(--surface-2)'
              el.style.borderColor = 'var(--hairline)'
            }}
            onMouseLeave={e => {
              const el = e.currentTarget as HTMLElement
              el.style.background = ''
              el.style.borderColor = 'transparent'
            }}
          >
            <div style={{ width: 8, height: 8, borderRadius: '50%', background: CURATED_PALETTE[i % CURATED_PALETTE.length], flexShrink: 0 }} />
            <div style={{ flex: 1, fontSize: 13, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {d.category}
            </div>
            <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text)', fontVariantNumeric: 'tabular-nums' }}>
              {formatINR(d.amount)}
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-2)', width: 42, textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
              {d.pct.toFixed(1)}%
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
