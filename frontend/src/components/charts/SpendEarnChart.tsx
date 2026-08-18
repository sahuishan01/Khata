import { Bar, BarChart, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { formatINR } from '../../utils/format'

interface MonthBucket { month: string; spent: number; earned: number }

function CustomTooltip({ active, payload, label }: any) {
  if (active && payload && payload.length) {
    return (
      <div style={{
        background: 'var(--surface-2)',
        border: '1px solid var(--hairline)',
        borderRadius: 8,
        padding: '10px 14px',
        boxShadow: '0 8px 24px rgba(0,0,0,0.4)',
        backdropFilter: 'blur(8px)',
      }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-2)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 6 }}>
          {label}
        </div>
        {payload.map((p: any, idx: number) => (
          <div key={idx} style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, marginTop: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: '50%', background: p.color }} />
            <span style={{ color: 'var(--text-2)', flex: 1 }}>{p.name}:</span>
            <span style={{ fontWeight: 600, color: 'var(--text)', fontVariantNumeric: 'tabular-nums' }}>
              {formatINR(Number(p.value))}
            </span>
          </div>
        ))}
      </div>
    )
  }
  return null
}

export function SpendEarnChart({ data }: { data: MonthBucket[] }) {
  return (
    <ResponsiveContainer width="100%" height={270}>
      <BarChart data={[...data].reverse()} margin={{ top: 10, right: 10, left: -10, bottom: 0 }}>
        <XAxis
          dataKey="month"
          tick={{ fontSize: 11, fill: 'var(--text-2)' }}
          axisLine={{ stroke: 'var(--hairline)' }}
          tickLine={false}
        />
        <YAxis
          tickFormatter={(v: number) => formatINR(v)}
          tick={{ fontSize: 11, fill: 'var(--text-2)' }}
          width={75}
          axisLine={false}
          tickLine={false}
        />
        <Tooltip content={<CustomTooltip />} cursor={{ fill: 'rgba(255,255,255,0.03)' }} />
        <Legend
          verticalAlign="top"
          align="right"
          wrapperStyle={{ paddingBottom: 12, fontSize: 12, color: 'var(--text-2)' }}
        />
        <Bar dataKey="spent" fill="var(--expense)" name="Spent" radius={[6, 6, 0, 0]} maxBarSize={32} />
        <Bar dataKey="earned" fill="var(--income)" name="Earned" radius={[6, 6, 0, 0]} maxBarSize={32} />
      </BarChart>
    </ResponsiveContainer>
  )
}
