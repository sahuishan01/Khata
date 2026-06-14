import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, TrendingUp, TrendingDown, Wallet } from 'lucide-react'
import { Screen, Card, CardBody, ListRow, ListRowText, StatCard, Amount, Field, Select, Chip, Button, EmptyState } from '../components/shared'

interface Asset { id: string; name: string; asset_type: string; value: number; recorded_at: string }
interface Liability { id: string; name: string; liability_type: string; value: number; recorded_at: string }
interface Snapshot { total_assets: number; total_liabilities: number; net_worth: number; assets: Asset[]; liabilities: Liability[] }

const ASSET_TYPES = ['bank', 'mutual_fund', 'stock', 'fd', 'cash', 'other']
const LIABILITY_TYPES = ['loan', 'credit_card', 'other']

export function PortfolioPage() {
  const [snap, setSnap] = useState<Snapshot | null>(null)
  const [assetName, setAssetName] = useState('')
  const [assetType, setAssetType] = useState('bank')
  const [assetValue, setAssetValue] = useState('')
  const [liabName, setLiabName] = useState('')
  const [liabType, setLiabType] = useState('loan')
  const [liabValue, setLiabValue] = useState('')
  const [error, setError] = useState('')

  const load = () => api.get<Snapshot>('/portfolio/snapshot').then(r => setSnap(r.data)).catch(() => {})
  useEffect(() => { load() }, [])

  const addAsset = async (e: React.FormEvent) => {
    e.preventDefault(); setError('')
    try { await api.post('/portfolio/assets', { name: assetName, asset_type: assetType, value: parseFloat(assetValue) }); setAssetName(''); setAssetValue(''); await load() }
    catch (err: unknown) { const e = err as { response?: { data?: { error?: string } } }; setError(e.response?.data?.error ?? 'Failed') }
  }
  const delAsset = async (id: string) => { try { await api.delete(`/portfolio/assets/${id}`); await load() } catch { setError('Failed') } }
  const addLiab = async (e: React.FormEvent) => {
    e.preventDefault(); setError('')
    try { await api.post('/portfolio/liabilities', { name: liabName, liability_type: liabType, value: parseFloat(liabValue) }); setLiabName(''); setLiabValue(''); await load() }
    catch (err: unknown) { const e = err as { response?: { data?: { error?: string } } }; setError(e.response?.data?.error ?? 'Failed') }
  }
  const delLiab = async (id: string) => { try { await api.delete(`/portfolio/liabilities/${id}`); await load() } catch { setError('Failed') } }

  return (
    <Screen title="Portfolio" subtitle="Track your net worth — assets minus liabilities">
      {snap && (
        <div className="grid grid-3" style={{ marginBottom: 16 }}>
          <StatCard label="Total Assets" value={<Amount paise={snap.total_assets} size="lg" />} icon={<TrendingUp size={16} />} color="income" />
          <StatCard label="Total Liabilities" value={<Amount paise={snap.total_liabilities} size="lg" />} icon={<TrendingDown size={16} />} color="expense" />
          <StatCard label="Net Worth" value={<Amount paise={snap.net_worth} size="lg" />} icon={<Wallet size={16} />} color={snap.net_worth >= 0 ? 'income' : 'expense'} />
        </div>
      )}

      {error && <p className="text-error" style={{ marginBottom: 16 }}>{error}</p>}

      <div className="grid grid-2" style={{ marginBottom: 16 }}>
        <Card>
          <CardBody>
            <form onSubmit={addAsset} style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 12 }}>
              <Field style={{ flex: 1, minWidth: 100 }} placeholder="Name" value={assetName} onChange={e => setAssetName(e.target.value)} required />
              <Select value={assetType} onChange={e => setAssetType(e.target.value)} style={{ width: 'auto' }}>{ASSET_TYPES.map(t => <option key={t} value={t}>{t.replace('_', ' ')}</option>)}</Select>
              <Field style={{ width: 100 }} type="number" step="0.01" placeholder="Value" value={assetValue} onChange={e => setAssetValue(e.target.value)} required />
              <Button size="sm"><Plus size={14} /></Button>
            </form>
            {snap?.assets.map(a => (
              <ListRow
                key={a.id}
                leading={<Chip color="green">{a.asset_type}</Chip>}
                trailing={
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <Amount paise={a.value} size="sm" />
                    <Button variant="ghost" size="sm" style={{ color: 'var(--expense)' }} onClick={() => delAsset(a.id)}><Trash2 size={13} /></Button>
                  </div>
                }
              >
                <ListRowText primary={a.name} />
              </ListRow>
            ))}
            {(snap?.assets.length ?? 0) === 0 && <EmptyState icon="💎" title="No assets yet" description="Add investments, property, or valuables to track your net worth." action={{ label: 'Add asset', onClick: () => document.querySelector<HTMLInputElement>('input[placeholder="Name"]')?.focus() }} />}
          </CardBody>
        </Card>

        <Card>
          <CardBody>
            <form onSubmit={addLiab} style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 12 }}>
              <Field style={{ flex: 1, minWidth: 100 }} placeholder="Name" value={liabName} onChange={e => setLiabName(e.target.value)} required />
              <Select value={liabType} onChange={e => setLiabType(e.target.value)} style={{ width: 'auto' }}>{LIABILITY_TYPES.map(t => <option key={t} value={t}>{t.replace('_', ' ')}</option>)}</Select>
              <Field style={{ width: 100 }} type="number" step="0.01" placeholder="Value" value={liabValue} onChange={e => setLiabValue(e.target.value)} required />
              <Button size="sm"><Plus size={14} /></Button>
            </form>
            {snap?.liabilities.map(l => (
              <ListRow
                key={l.id}
                leading={<Chip color="red">{l.liability_type}</Chip>}
                trailing={
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <Amount paise={l.value} size="sm" />
                    <Button variant="ghost" size="sm" style={{ color: 'var(--expense)' }} onClick={() => delLiab(l.id)}><Trash2 size={13} /></Button>
                  </div>
                }
              >
                <ListRowText primary={l.name} />
              </ListRow>
            ))}
            {(snap?.liabilities.length ?? 0) === 0 && <EmptyState icon="📋" title="No liabilities yet" description="Add loans, credit card debt, or other obligations." action={{ label: 'Add liability', onClick: () => document.querySelector<HTMLInputElement>('input[placeholder="Name"]')?.focus() }} />}
          </CardBody>
        </Card>
      </div>
    </Screen>
  )
}
