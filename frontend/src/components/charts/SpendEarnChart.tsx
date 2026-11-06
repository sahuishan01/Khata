import { Bar, BarChart, CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { formatINR } from '../../utils/format'
import { ChartTooltip, TooltipRow } from './ChartTooltip'

interface MonthBucket { month: string; spent: number; earned: number }

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

/** "2026-03" → "Mar 26" */
function monthLabel(ym: string) {
  const [y, m] = ym.split('-')
  const i = Number(m) - 1
  return i >= 0 && i < 12 ? `${MONTHS[i]} ${y.slice(2)}` : ym
}

/** Compact axis ticks in Indian units: ₹1.2L, ₹3.4Cr, ₹45K. */
function compactINR(v: number) {
  const a = Math.abs(v)
  if (a >= 1e7) return `₹${(v / 1e7).toFixed(1).replace(/\.0$/, '')}Cr`
  if (a >= 1e5) return `₹${(v / 1e5).toFixed(1).replace(/\.0$/, '')}L`
  if (a >= 1e3) return `₹${Math.round(v / 1e3)}K`
  return `₹${v}`
}

function SpendEarnTooltip({ active, payload, label }: any) {
  if (!active || !payload?.length) return null
  return (
    <ChartTooltip title={monthLabel(String(label))}>
      {payload.map((p: any) => (
        <TooltipRow key={p.dataKey} swatch={p.color} label={p.name} value={formatINR(Number(p.value))} />
      ))}
    </ChartTooltip>
  )
}

/** Money in vs money out per month — green/red carry their app-wide meaning. */
export function SpendEarnChart({ data }: { data: MonthBucket[] }) {
  return (
    <ResponsiveContainer width="100%" height={260}>
      <BarChart data={[...data].reverse()} margin={{ top: 4, right: 4, left: 0, bottom: 0 }} barGap={2}>
        <CartesianGrid vertical={false} stroke="var(--hairline)" />
        <XAxis
          dataKey="month"
          tickFormatter={monthLabel}
          tick={{ fontSize: 11, fill: 'var(--text-2)', fontFamily: 'var(--font-mono)' }}
          axisLine={{ stroke: 'var(--rule)' }}
          tickLine={false}
        />
        <YAxis
          tickFormatter={compactINR}
          tick={{ fontSize: 11, fill: 'var(--text-2)', fontFamily: 'var(--font-mono)' }}
          width={56}
          axisLine={false}
          tickLine={false}
        />
        <Tooltip content={<SpendEarnTooltip />} cursor={{ fill: 'rgba(241, 236, 227, 0.04)' }} />
        <Legend
          verticalAlign="top"
          align="right"
          iconType="square"
          iconSize={8}
          wrapperStyle={{ paddingBottom: 12, fontSize: 12, color: 'var(--text-2)' }}
        />
        <Bar dataKey="earned" fill="var(--income)" name="Earned" radius={[4, 4, 0, 0]} maxBarSize={20} isAnimationActive={false} />
        <Bar dataKey="spent" fill="var(--expense)" name="Spent" radius={[4, 4, 0, 0]} maxBarSize={20} isAnimationActive={false} />
      </BarChart>
    </ResponsiveContainer>
  )
}
