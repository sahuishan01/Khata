import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts'
import { formatINR } from '../../utils/format'
import { ChartTooltip, TooltipRow } from './ChartTooltip'
import { colorFor, foldToOther } from './palette'

interface CategoryBucket { category: string; amount: number; pct: number }

interface Props {
  data: CategoryBucket[]
  onCategoryClick?: (category: string) => void
}

function CategoryTooltip({ active, payload }: any) {
  if (!active || !payload?.length) return null
  const d: CategoryBucket & { fill?: string } = payload[0].payload
  return (
    <ChartTooltip>
      <TooltipRow swatch={payload[0].payload.fill ?? 'var(--text-2)'} label={d.category} value={formatINR(d.amount)} />
      <div className="mono" style={{ fontSize: 12, color: 'var(--text-2)', marginTop: 4, textAlign: 'right' }}>{d.pct.toFixed(1)}% of spend</div>
    </ChartTooltip>
  )
}

export function CategoryChart({ data, onCategoryClick }: Props) {
  const folded = foldToOther(data).map((d, i) => ({ ...d, fill: colorFor(d.category, i) }))
  const total = data.reduce((acc, d) => acc + d.amount, 0)
  const clickable = (c: string) => onCategoryClick && c !== 'Other'

  return (
    <div style={{ display: 'flex', gap: 24, alignItems: 'center', flexWrap: 'wrap' }}>
      <div style={{ position: 'relative', width: 180, height: 180, flexShrink: 0, margin: '0 auto' }}>
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={folded}
              dataKey="amount"
              nameKey="category"
              cx="50%" cy="50%"
              innerRadius={60} outerRadius={86}
              paddingAngle={1}
              isAnimationActive={false}
              stroke="var(--surface)" strokeWidth={2}
              onClick={onCategoryClick ? (entry) => {
                const c = (entry as unknown as CategoryBucket).category
                if (clickable(c)) onCategoryClick(c)
              } : undefined}
              style={onCategoryClick ? { cursor: 'pointer' } : undefined}
            >
              {folded.map(d => <Cell key={d.category} fill={d.fill} />)}
            </Pie>
            <Tooltip content={<CategoryTooltip />} />
          </PieChart>
        </ResponsiveContainer>
        <div style={{
          position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column',
          alignItems: 'center', justifyContent: 'center', pointerEvents: 'none', textAlign: 'center',
        }}>
          <span style={{ fontSize: 12, color: 'var(--text-2)' }}>Spent</span>
          <span className="figure" style={{ fontSize: 15, color: 'var(--text)', marginTop: 2 }}>{formatINR(total)}</span>
        </div>
      </div>

      {/* Legend doubles as the table view: name, amount, share. */}
      <div style={{ flex: 1, minWidth: 200 }}>
        {folded.map(d => {
          const canClick = clickable(d.category)
          const Row = canClick ? 'button' : 'div'
          return (
            <Row
              key={d.category}
              className={canClick ? 'list-row' : undefined}
              onClick={canClick ? () => onCategoryClick!(d.category) : undefined}
              style={{
                display: 'flex', alignItems: 'center', gap: 10, width: '100%',
                minHeight: 36, padding: '0 8px', margin: 0,
                background: 'transparent', border: 'none', borderRadius: 'var(--r-sm)',
                color: 'inherit', font: 'inherit', textAlign: 'left',
                cursor: canClick ? 'pointer' : 'default',
              }}
            >
              <span style={{ width: 10, height: 10, borderRadius: 2, background: d.fill, flexShrink: 0 }} />
              <span style={{ flex: 1, fontSize: 14, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {d.category}
              </span>
              <span className="figure" style={{ fontSize: 13, color: 'var(--text)' }}>{formatINR(d.amount)}</span>
              <span className="mono" style={{ fontSize: 12, color: 'var(--text-2)', width: 48, textAlign: 'right' }}>
                {d.pct.toFixed(1)}%
              </span>
            </Row>
          )
        })}
      </div>
    </div>
  )
}
