import { Bar, BarChart, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { formatINR } from '../../utils/format'

interface MonthBucket { month: string; spent: number; earned: number }

export function SpendEarnChart({ data }: { data: MonthBucket[] }) {

  return (
    <ResponsiveContainer width="100%" height={260}>
      <BarChart data={[...data].reverse()}>
        <XAxis dataKey="month" tick={{ fontSize: 11 }} />
        <YAxis tickFormatter={(v: number) => formatINR(v)} tick={{ fontSize: 11 }} width={80} />
        <Tooltip formatter={(v) => formatINR(Number(v))} />
        <Legend />
        <Bar dataKey="spent"  fill="#ef4444" name="Spent"  />
        <Bar dataKey="earned" fill="#22c55e" name="Earned" />
      </BarChart>
    </ResponsiveContainer>
  )
}
