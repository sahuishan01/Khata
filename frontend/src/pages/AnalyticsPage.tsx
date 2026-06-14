import { useState, useEffect, useCallback } from 'react'
import { api } from '../api/client'
import { Screen, Card, CardHeader, CardBody, Button, Pill, Chip, Amount, StatCard, EmptyState } from '../components/shared'
import { BarChart as BarChartIcon, PieChart, TrendingUp, Download, SlidersHorizontal } from 'lucide-react'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'
import { CategoryChart } from '../components/charts/CategoryChart'
import { formatINR } from '../utils/format'

type GroupBy = 'category' | 'payee' | 'account' | 'month' | 'week'
type Dimension = 'spent' | 'earned' | 'net'
type Compare = 'mom' | 'yoy' | null
type WidgetKey = 'monthly_trend' | 'category_breakdown' | 'top_payees' | 'comparison' | 'insights' | 'budget_vs_actual' | 'savings_rate'

interface WidgetConfig { key: WidgetKey; label: string; visible: boolean; sortOrder: number }

const DEFAULT_WIDGETS: WidgetConfig[] = [
  { key: 'monthly_trend', label: 'Monthly Trend', visible: true, sortOrder: 0 },
  { key: 'category_breakdown', label: 'Category Breakdown', visible: true, sortOrder: 1 },
  { key: 'comparison', label: 'Month Comparison', visible: true, sortOrder: 2 },
  { key: 'top_payees', label: 'Top Payees', visible: true, sortOrder: 3 },
  { key: 'insights', label: 'Insights', visible: true, sortOrder: 4 },
  { key: 'budget_vs_actual', label: 'Budget vs Actual', visible: false, sortOrder: 5 },
  { key: 'savings_rate', label: 'Savings Rate Trend', visible: false, sortOrder: 6 },
]

export function AnalyticsPage() {
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')
  const [preset, setPreset] = useState<string>('year')
  const [groupBy, setGroupBy] = useState<GroupBy>('category')
  const [dimension, setDimension] = useState<Dimension>('spent')
  const [compare, setCompare] = useState<Compare>(null)
  const [categoryFilter, setCategoryFilter] = useState('')
  const [widgets, setWidgets] = useState<WidgetConfig[]>(DEFAULT_WIDGETS)
  const [editMode, setEditMode] = useState(false)
  const [series, setSeries] = useState<{ labels: string[]; values: number[]; total: number } | null>(null)
  const [budgets, setBudgets] = useState<any[]>([])
  const [stats, setStats] = useState<any>(null)
  const [analysis, setAnalysis] = useState<any>(null)

  const datePresets: { key: string; label: string; fn: () => { from: string; to: string } }[] = [
    { key: 'month', label: 'This month', fn: () => { const d = new Date(); return { from: new Date(d.getFullYear(), d.getMonth(), 1).toISOString().slice(0,10), to: d.toISOString().slice(0,10) } } },
    { key: 'quarter', label: 'This quarter', fn: () => { const d = new Date(); const q = Math.floor(d.getMonth() / 3) * 3; return { from: new Date(d.getFullYear(), q, 1).toISOString().slice(0,10), to: d.toISOString().slice(0,10) } } },
    { key: 'year', label: 'This year', fn: () => { const d = new Date(); return { from: new Date(d.getFullYear(), 0, 1).toISOString().slice(0,10), to: d.toISOString().slice(0,10) } } },
    { key: 'all', label: 'All time', fn: () => ({ from: '', to: '' }) },
  ]

  const applyPreset = useCallback((key: string) => {
    setPreset(key)
    const p = datePresets.find(d => d.key === key)
    if (p) { const r = p.fn(); setFrom(r.from); setTo(r.to) }
  }, [])

  useEffect(() => { applyPreset('year') }, [])

  const fetchSeries = useCallback(async () => {
    try {
      const params = new URLSearchParams()
      if (from) params.set('from', from)
      if (to) params.set('to', to)
      params.set('group_by', groupBy)
      params.set('dimension', dimension)
      const { data } = await api.get(`/txns/analytics/explore?${params}`)
      setSeries(data.series)
    } catch {}
  }, [from, to, groupBy, dimension])

  useEffect(() => { fetchSeries() }, [fetchSeries])

  useEffect(() => {
    Promise.all([
      api.get('/txns/analysis').catch(() => ({ data: null })),
      api.get('/budgets').catch(() => ({ data: [] })),
    ]).then(([a, b]) => { setAnalysis(a.data); setBudgets(b.data) })
  }, [])

  const toggleWidget = (key: WidgetKey) => {
    setWidgets(w => w.map(w => w.key === key ? { ...w, visible: !w.visible } : w))
  }

  const moveWidget = (key: WidgetKey, dir: -1 | 1) => {
    setWidgets(w => {
      const idx = w.findIndex(x => x.key === key)
      if (idx === -1 || (dir === -1 && idx === 0) || (dir === 1 && idx === w.length - 1)) return w
      const next = [...w]
      ;[next[idx], next[idx + dir]] = [next[idx + dir], next[idx]]
      return next.map((x, i) => ({ ...x, sortOrder: i }))
    })
  }

  const visibleWidgets = widgets.filter(w => editMode || w.visible).sort((a, b) => a.sortOrder - b.sortOrder)

  return (
    <Screen
      title="Analytics"
      subtitle="Deep analysis of your finances"
      actions={
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <Button variant="ghost" size="sm" onClick={() => setEditMode(!editMode)}>
            <SlidersHorizontal size={14} /> {editMode ? 'Done' : 'Edit layout'}
          </Button>
          <Button variant="ghost" size="sm"><Download size={14} /> Export</Button>
        </div>
      }
    >
      {/* Global filter bar */}
      <Card style={{ marginBottom: 16 }}>
        <CardBody>
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
            {datePresets.map(p => (
              <Pill key={p.key} active={preset === p.key} onClick={() => applyPreset(p.key)}>{p.label}</Pill>
            ))}
            <input type="date" className="form-input" style={{ width: 'auto', padding: '4px 8px', fontSize: 12 }} value={from} onChange={e => { setFrom(e.target.value); setPreset('custom') }} />
            <span style={{ color: 'var(--text-2)', fontSize: 12 }}>to</span>
            <input type="date" className="form-input" style={{ width: 'auto', padding: '4px 8px', fontSize: 12 }} value={to} onChange={e => { setTo(e.target.value); setPreset('custom') }} />

            <select className="form-input" style={{ width: 'auto', padding: '4px 8px', fontSize: 12 }} value={groupBy} onChange={e => setGroupBy(e.target.value as GroupBy)}>
              <option value="category">Group: Category</option>
              <option value="payee">Group: Payee</option>
              <option value="account">Group: Account</option>
              <option value="month">Group: Month</option>
              <option value="week">Group: Week</option>
            </select>

            <select className="form-input" style={{ width: 'auto', padding: '4px 8px', fontSize: 12 }} value={dimension} onChange={e => setDimension(e.target.value as Dimension)}>
              <option value="spent">Spending</option>
              <option value="earned">Income</option>
              <option value="net">Net</option>
            </select>

            <Pill active={compare === 'mom'} onClick={() => setCompare(compare === 'mom' ? null : 'mom')}>MoM</Pill>
            <Pill active={compare === 'yoy'} onClick={() => setCompare(compare === 'yoy' ? null : 'yoy')}>YoY</Pill>
          </div>
        </CardBody>
      </Card>

      {/* Edit layout mode */}
      {editMode && (
        <Card style={{ marginBottom: 16, borderColor: 'var(--brand)' }}>
          <CardHeader title="CUSTOMISE LAYOUT" />
          <CardBody>
            {widgets.map((w, i) => (
              <div key={w.key} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 0' }}>
                <div style={{ display: 'flex', gap: 4 }}>
                  <Button variant="ghost" size="sm" disabled={i === 0} onClick={() => moveWidget(w.key, -1)}>↑</Button>
                  <Button variant="ghost" size="sm" disabled={i === widgets.length - 1} onClick={() => moveWidget(w.key, 1)}>↓</Button>
                </div>
                <label style={{ flex: 1, display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer', fontSize: 13 }}>
                  <input type="checkbox" checked={w.visible} onChange={() => toggleWidget(w.key)} />
                  {w.label}
                </label>
              </div>
            ))}
          </CardBody>
        </Card>
      )}

      {/* Widgets */}
      {visibleWidgets.map(w => {
        switch (w.key) {
          case 'monthly_trend':
            return (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="MONTHLY TREND" action={<Chip color="gray">{dimension}</Chip>} />
                <CardBody>
                  {stats?.monthly ? <SpendEarnChart data={stats.monthly} /> : <EmptyState icon="📊" title="No data" description="Upload statements to see trends" />}
                </CardBody>
              </Card>
            )

          case 'category_breakdown':
            return (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title={`${dimension === 'earned' ? 'INCOME' : 'SPENDING'} BY CATEGORY`} />
                <CardBody>
                  {analysis?.category_breakdown ? (
                    <CategoryChart data={analysis.category_breakdown} onCategoryClick={c => setCategoryFilter(c)} />
                  ) : <EmptyState icon="📊" title="No data" description="Upload statements to see breakdowns" />}
                </CardBody>
              </Card>
            )

          case 'comparison':
            return (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="MONTH COMPARISON" />
                <CardBody>
                  {analysis?.month_comparison ? (
                    <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap' }}>
                      <div>
                        <div className="text-muted" style={{ fontSize: 11, marginBottom: 4 }}>This month</div>
                        <Amount paise={analysis.month_comparison.this_month} size="lg" />
                      </div>
                      <div>
                        <div className="text-muted" style={{ fontSize: 11, marginBottom: 4 }}>Last month</div>
                        <Amount paise={analysis.month_comparison.last_month} size="lg" />
                      </div>
                      <div>
                        <div className="text-muted" style={{ fontSize: 11, marginBottom: 4 }}>Change</div>
                        <div style={{ fontSize: 22, fontWeight: 700, fontVariantNumeric: 'tabular-nums', color: analysis.month_comparison.change_pct > 0 ? 'var(--expense)' : 'var(--income)' }}>
                          {analysis.month_comparison.change_pct > 0 ? '+' : ''}{analysis.month_comparison.change_pct.toFixed(1)}%
                        </div>
                      </div>
                    </div>
                  ) : <EmptyState icon="📊" title="No data" description="Upload statements to see comparisons" />}
                </CardBody>
              </Card>
            )

          case 'top_payees':
            return (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="TOP PAYEES" />
                <CardBody>
                  {series ? (
                    series.labels.slice(0, 10).map((label, i) => (
                      <div key={label} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 0', borderBottom: i < Math.min(series.labels.length, 10) - 1 ? '1px solid var(--hairline)' : 'none' }}>
                        <span style={{ fontSize: 13, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '70%' }}>{label}</span>
                        <Amount paise={series.values[i]} size="sm" />
                      </div>
                    ))
                  ) : <EmptyState icon="📄" title="No data" description="Upload statements to see top payees" />}
                </CardBody>
              </Card>
            )

          case 'insights':
            return (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="INSIGHTS" />
                <CardBody>
                  {analysis ? (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                      {analysis.savings_rate_pct !== undefined && (
                        <div style={{ padding: '10px 12px', background: 'var(--brand-soft)', borderRadius: 8, fontSize: 13 }}>
                          Savings rate: <strong>{analysis.savings_rate_pct.toFixed(0)}%</strong> of income
                        </div>
                      )}
                      {analysis.month_comparison && (
                        <div style={{ padding: '10px 12px', background: analysis.month_comparison.change_pct > 0 ? 'var(--expense-soft)' : 'var(--income-soft)', borderRadius: 8, fontSize: 13 }}>
                          Spending {analysis.month_comparison.change_pct > 0 ? 'up' : 'down'} <strong>{Math.abs(analysis.month_comparison.change_pct).toFixed(0)}%</strong> vs last month
                        </div>
                      )}
                    </div>
                  ) : <EmptyState icon="💡" title="No insights yet" description="Upload statements to get insights" />}
                </CardBody>
              </Card>
            )

          default:
            return null
        }
      })}
    </Screen>
  )
}
