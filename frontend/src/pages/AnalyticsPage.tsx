import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { Screen, Card, CardHeader, CardBody, Button, Pill, Chip, Amount, EmptyState } from '../components/shared'
import { SlidersHorizontal, TrendingUp, TrendingDown, Zap, Star, ArrowRight } from 'lucide-react'
import { SpendEarnChart } from '../components/charts/SpendEarnChart'
import { CategoryChart } from '../components/charts/CategoryChart'

type GroupBy = 'category' | 'payee' | 'account' | 'month' | 'week'
type Dimension = 'spent' | 'earned' | 'net'
type WidgetKey = 'monthly_trend' | 'category_breakdown' | 'top_payees' | 'comparison' | 'highlights' | 'insights'

interface WidgetConfig { key: WidgetKey; label: string; visible: boolean; sortOrder: number }

const DEFAULT_WIDGETS: WidgetConfig[] = [
  { key: 'highlights', label: 'Highlights', visible: true, sortOrder: 0 },
  { key: 'monthly_trend', label: 'Monthly Trend', visible: true, sortOrder: 1 },
  { key: 'category_breakdown', label: 'Category Breakdown', visible: true, sortOrder: 2 },
  { key: 'comparison', label: 'Month Comparison', visible: true, sortOrder: 3 },
  { key: 'top_payees', label: 'Top Payees', visible: true, sortOrder: 4 },
  { key: 'insights', label: 'Insights', visible: true, sortOrder: 5 },
]

export function AnalyticsPage() {
  const navigate = useNavigate()
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')
  const [preset, setPreset] = useState<string>('year')
  const [groupBy, setGroupBy] = useState<GroupBy>('category')
  const [dimension, setDimension] = useState<Dimension>('spent')
  const [widgets, setWidgets] = useState<WidgetConfig[]>(DEFAULT_WIDGETS)
  const [editMode, setEditMode] = useState(false)
  const [series, setSeries] = useState<{ labels: string[]; values: number[]; total: number } | null>(null)
  const [stats, setStats] = useState<any>(null)
  const [analysis, setAnalysis] = useState<any>(null)
  const [highlights, setHighlights] = useState<any>(null)

  // Comparison period state
  const [p1From, setP1From] = useState('')
  const [p1To, setP1To] = useState('')
  const [p2From, setP2From] = useState('')
  const [p2To, setP2To] = useState('')
  const [comparisonData, setComparisonData] = useState<any>(null)

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
      api.get('/txns/analytics/highlights').catch(() => ({ data: null })),
    ]).then(([a, h]) => { setAnalysis(a.data); setHighlights(h.data) })

    // Default comparison: this month vs last month
    const now = new Date()
    const thisStart = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().slice(0,10)
    const thisEnd = now.toISOString().slice(0,10)
    const lastStart = new Date(now.getFullYear(), now.getMonth() - 1, 1).toISOString().slice(0,10)
    const lastEnd = new Date(now.getFullYear(), now.getMonth(), 0).toISOString().slice(0,10)
    setP1From(thisStart); setP1To(thisEnd)
    setP2From(lastStart); setP2To(lastEnd)
  }, [])

  const fetchComparison = useCallback(async () => {
    try {
      const [p1, p2] = await Promise.all([
        api.get(`/txns/analytics/explore?from=${p1From}&to=${p1To}&group_by=category&dimension=spent`).catch(() => ({ data: null })),
        api.get(`/txns/analytics/explore?from=${p2From}&to=${p2To}&group_by=category&dimension=spent`).catch(() => ({ data: null })),
      ])
      setComparisonData({ period1: p1.data?.series, period2: p2.data?.series })
    } catch {}
  }, [p1From, p1To, p2From, p2To])

  useEffect(() => { if (p1From && p2From) fetchComparison() }, [fetchComparison])

  const toggleWidget = (key: WidgetKey) => {
    setWidgets(w => w.map(w => w.key === key ? { ...w, visible: !w.visible } : w))
  }

  const moveWidget = (key: WidgetKey, dir: -1 | 1) => {
    setWidgets(w => {
      const idx = w.findIndex(x => x.key === key)
      if (idx === -1 || (dir === -1 && idx === 0) || (dir === 1 && idx === w.length - 1)) return w
      const next = [...w]; [next[idx], next[idx + dir]] = [next[idx + dir], next[idx]]
      return next.map((x, i) => ({ ...x, sortOrder: i }))
    })
  }

  const visibleWidgets = widgets.filter(w => editMode || w.visible).sort((a, b) => a.sortOrder - b.sortOrder)

  return (
    <Screen title="Analytics" subtitle="Deep analysis of your finances" actions={
      <Button variant="ghost" size="sm" onClick={() => setEditMode(!editMode)}>
        <SlidersHorizontal size={14} /> {editMode ? 'Done' : 'Edit layout'}
      </Button>
    }>
      {/* Global filter bar */}
      <Card style={{ marginBottom: 16 }}>
        <CardBody>
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
            {datePresets.map(p => <Pill key={p.key} active={preset === p.key} onClick={() => applyPreset(p.key)}>{p.label}</Pill>)}
            <input type="date" style={{ width: 'auto', padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={from} onChange={e => { setFrom(e.target.value); setPreset('custom') }} />
            <span style={{ color: 'var(--text-2)', fontSize: 12 }}>to</span>
            <input type="date" style={{ width: 'auto', padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={to} onChange={e => { setTo(e.target.value); setPreset('custom') }} />
            <select style={{ width: 'auto', padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={groupBy} onChange={e => setGroupBy(e.target.value as GroupBy)}>
              <option value="category">Group: Category</option>
              <option value="payee">Group: Payee</option>
              <option value="month">Group: Month</option>
            </select>
            <select style={{ width: 'auto', padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={dimension} onChange={e => setDimension(e.target.value as Dimension)}>
              <option value="spent">Spending</option>
              <option value="earned">Income</option>
              <option value="net">Net</option>
            </select>
          </div>
        </CardBody>
      </Card>

      {/* Edit layout */}
      {editMode && (
        <Card style={{ marginBottom: 16, borderColor: 'var(--brand)' }}>
          <CardHeader title="CUSTOMISE LAYOUT" />
          <CardBody>
            {widgets.map((w, i) => (
              <div key={w.key} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 0' }}>
                <Button variant="ghost" size="sm" disabled={i === 0} onClick={() => moveWidget(w.key, -1)}>↑</Button>
                <Button variant="ghost" size="sm" disabled={i === widgets.length - 1} onClick={() => moveWidget(w.key, 1)}>↓</Button>
                <label style={{ flex: 1, display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer', fontSize: 13 }}>
                  <input type="checkbox" checked={w.visible} onChange={() => toggleWidget(w.key)} /> {w.label}
                </label>
              </div>
            ))}
          </CardBody>
        </Card>
      )}

      {/* Widgets */}
      {visibleWidgets.map(w => {
        switch (w.key) {
          case 'highlights':
            return highlights && (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="HIGHLIGHTS" />
                <CardBody>
                  <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))', gap: 8 }}>
                    {highlights.highest_spending_month && (
                      <div className="more-card" onClick={() => navigate(`/analytics/detail?month=${highlights.highest_spending_month.month}`)}>
                        <TrendingUp size={18} style={{ color: 'var(--expense)' }} />
                        <span style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>Highest spend</span>
                        <Amount paise={highlights.highest_spending_month.spent} size="sm" />
                        <span style={{ fontSize: 10, color: 'var(--text-muted)' }}>{highlights.highest_spending_month.month}</span>
                      </div>
                    )}
                    {highlights.highest_earning_month && (
                      <div className="more-card" onClick={() => navigate(`/analytics/detail?month=${highlights.highest_earning_month.month}`)}>
                        <TrendingDown size={18} style={{ color: 'var(--income)' }} />
                        <span style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>Highest income</span>
                        <Amount paise={highlights.highest_earning_month.earned} size="sm" />
                        <span style={{ fontSize: 10, color: 'var(--text-muted)' }}>{highlights.highest_earning_month.month}</span>
                      </div>
                    )}
                    {highlights.biggest_expense && (
                      <div className="more-card" onClick={() => navigate(`/analytics/detail?payee=${encodeURIComponent(highlights.biggest_expense.description)}`)}>
                        <Zap size={18} style={{ color: 'var(--expense)' }} />
                        <span style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>Biggest expense</span>
                        <Amount paise={highlights.biggest_expense.amount} size="sm" />
                        <span style={{ fontSize: 10, color: 'var(--text-muted)', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '100%' }}>{highlights.biggest_expense.description}</span>
                      </div>
                    )}
                    {highlights.top_payee && (
                      <div className="more-card" onClick={() => navigate(`/analytics/detail?payee=${encodeURIComponent(highlights.top_payee.description)}`)}>
                        <Star size={18} style={{ color: 'var(--warn)' }} />
                        <span style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>Top payee</span>
                        <Amount paise={highlights.top_payee.total} size="sm" />
                        <span style={{ fontSize: 10, color: 'var(--text-muted)', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '100%' }}>{highlights.top_payee.description}</span>
                      </div>
                    )}
                    {highlights.top_category && (
                      <div className="more-card" onClick={() => navigate(`/analytics/detail?category=${encodeURIComponent(highlights.top_category.category)}`)}>
                        <Star size={18} style={{ color: 'var(--brand)' }} />
                        <span style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>Top category</span>
                        <Amount paise={highlights.top_category.amount} size="sm" />
                        <span style={{ fontSize: 10, color: 'var(--text-muted)' }}>{highlights.top_category.category}</span>
                      </div>
                    )}
                  </div>
                </CardBody>
              </Card>
            )

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
                    <CategoryChart data={analysis.category_breakdown} onCategoryClick={c => navigate(`/analytics/detail?category=${encodeURIComponent(c)}`)} />
                  ) : <EmptyState icon="📊" title="No data" description="Upload statements" />}
                </CardBody>
              </Card>
            )

          case 'comparison':
            return (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="MONTH COMPARISON" />
                <CardBody>
                  <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 12 }}>
                    <div>
                      <div style={{ fontSize: 10, color: 'var(--text-muted)', marginBottom: 2 }}>Period 1</div>
                      <input type="date" style={{ width: 140, padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={p1From} onChange={e => setP1From(e.target.value)} />
                      <span style={{ color: 'var(--text-2)', fontSize: 11, margin: '0 4px' }}>→</span>
                      <input type="date" style={{ width: 140, padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={p1To} onChange={e => setP1To(e.target.value)} />
                    </div>
                    <div>
                      <div style={{ fontSize: 10, color: 'var(--text-muted)', marginBottom: 2 }}>Period 2</div>
                      <input type="date" style={{ width: 140, padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={p2From} onChange={e => setP2From(e.target.value)} />
                      <span style={{ color: 'var(--text-2)', fontSize: 11, margin: '0 4px' }}>→</span>
                      <input type="date" style={{ width: 140, padding: '4px 8px', fontSize: 12, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, color: 'var(--text)' }} value={p2To} onChange={e => setP2To(e.target.value)} />
                    </div>
                  </div>
                  {comparisonData && comparisonData.period1 && comparisonData.period2 ? (
                    <>
                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 8, marginBottom: 12 }}>
                        <div><div style={{ fontSize: 10, color: 'var(--text-muted)' }}>Period 1 total</div><Amount paise={comparisonData.period1.total} size="md" /></div>
                        <div><div style={{ fontSize: 10, color: 'var(--text-muted)' }}>Period 2 total</div><Amount paise={comparisonData.period2.total} size="md" /></div>
                        <div><div style={{ fontSize: 10, color: 'var(--text-muted)' }}>Change</div>
                          {(() => { const c = comparisonData.period2.total ? (comparisonData.period1.total - comparisonData.period2.total) / comparisonData.period2.total * 100 : 0; return (
                            <span style={{ fontSize: 18, fontWeight: 700, fontVariantNumeric: 'tabular-nums', color: c > 0 ? 'var(--expense)' : 'var(--income)' }}>
                              {c > 0 ? '+' : ''}{c.toFixed(1)}%
                            </span>
                          )})()}
                        </div>
                      </div>
                      {/* Category deltas */}
                      {comparisonData.period1.labels.length > 0 && <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 8 }}>Category movers</div>}
                      {comparisonData.period1.labels.slice(0, 8).map((label: string, i: number) => {
                        const v2 = comparisonData.period2.labels.indexOf(label) >= 0 ? comparisonData.period2.values[comparisonData.period2.labels.indexOf(label)] : 0
                        const v1 = comparisonData.period1.values[i]
                        const change = v2 ? (v1 - v2) / v2 * 100 : 100
                        return (
                          <div key={label} onClick={() => navigate(`/analytics/detail?category=${encodeURIComponent(label)}`)} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '5px 0', borderBottom: '1px solid var(--hairline)', cursor: 'pointer' }}>
                            <span style={{ flex: 1, fontSize: 12, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{label}</span>
                            <Amount paise={v1} size="sm" />
                            <ArrowRight size={10} style={{ color: 'var(--text-2)' }} />
                            <Amount paise={v2} size="sm" />
                            <span style={{ fontSize: 11, fontWeight: 600, color: change > 0 ? 'var(--expense)' : 'var(--income)', width: 60, textAlign: 'right' }}>
                              {change > 0 ? '+' : ''}{change.toFixed(0)}%
                            </span>
                          </div>
                        )
                      })}
                    </>
                  ) : <EmptyState icon="📊" title="Select periods" description="Pick two date ranges to compare" />}
                </CardBody>
              </Card>
            )

          case 'top_payees':
            return (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="TOP PAYEES" />
                <CardBody>
                  {series ? series.labels.slice(0, 10).map((label, i, arr) => (
                    <div key={label} onClick={() => navigate(`/analytics/detail?payee=${encodeURIComponent(label)}`)} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 0', borderBottom: i < arr.length - 1 ? '1px solid var(--hairline)' : 'none', cursor: 'pointer' }}>
                      <span style={{ fontSize: 13, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '70%' }}>{label}</span>
                      <Amount paise={series.values[i]} size="sm" />
                    </div>
                  )) : <EmptyState icon="📄" title="No data" description="Upload statements to see top payees" />}
                </CardBody>
              </Card>
            )

          case 'insights':
            return analysis && (
              <Card key={w.key} style={{ marginBottom: 12 }}>
                <CardHeader title="INSIGHTS" />
                <CardBody>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    {analysis.savings_rate_pct !== undefined && (
                      <div style={{ padding: '10px 12px', background: 'var(--brand-soft)', borderRadius: 8, fontSize: 13, cursor: 'pointer' }} onClick={() => navigate(`/analytics/detail?month=${new Date().toISOString().slice(0,7)}`)}>
                        Savings rate: <strong>{analysis.savings_rate_pct.toFixed(0)}%</strong> of income <ArrowRight size={12} style={{ verticalAlign: 'middle', marginLeft: 4 }} />
                      </div>
                    )}
                    {analysis.month_comparison && (
                      <div style={{ padding: '10px 12px', background: analysis.month_comparison.change_pct > 0 ? 'var(--expense-soft)' : 'var(--income-soft)', borderRadius: 8, fontSize: 13 }}>
                        Spending {analysis.month_comparison.change_pct > 0 ? 'up' : 'down'} <strong>{Math.abs(analysis.month_comparison.change_pct).toFixed(0)}%</strong> vs last month
                      </div>
                    )}
                  </div>
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
