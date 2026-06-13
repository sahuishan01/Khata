import { useState, useEffect } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, TrendingUp, TrendingDown, Wallet } from 'lucide-react'
import { formatINR } from '../utils/format'

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
    <div>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Portfolio</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Track your net worth — assets minus liabilities</p>

      {snap && (
        <div className="grid grid-3 mb-4">
          <div className="card" style={{ textAlign: 'center' }}>
            <TrendingUp size={18} style={{ color: 'var(--income)', marginBottom: 6 }} />
            <div className="stat-label">Total Assets</div>
            <div className="stat-value" style={{ fontSize: 18, color: 'var(--income)' }}>{formatINR(snap.total_assets)}</div>
          </div>
          <div className="card" style={{ textAlign: 'center' }}>
            <TrendingDown size={18} style={{ color: 'var(--expense)', marginBottom: 6 }} />
            <div className="stat-label">Total Liabilities</div>
            <div className="stat-value" style={{ fontSize: 18, color: 'var(--expense)' }}>{formatINR(snap.total_liabilities)}</div>
          </div>
          <div className="card" style={{ textAlign: 'center' }}>
            <Wallet size={18} style={{ color: 'var(--brand)', marginBottom: 6 }} />
            <div className="stat-label">Net Worth</div>
            <div className="stat-value" style={{ fontSize: 18, color: snap.net_worth >= 0 ? 'var(--income)' : 'var(--expense)' }}>{formatINR(snap.net_worth)}</div>
          </div>
        </div>
      )}

      {error && <p className="text-error mb-4">{error}</p>}

      <div className="grid grid-2 mb-4">
        <div className="card">
          <h2 style={{ fontSize: 15, fontWeight: 600, marginBottom: 12 }}>Assets</h2>
          <form onSubmit={addAsset} style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 12 }}>
            <input className="form-input" style={{ flex: 1, minWidth: 100 }} placeholder="Name" value={assetName} onChange={e => setAssetName(e.target.value)} required />
            <select className="form-input" style={{ width: 'auto' }} value={assetType} onChange={e => setAssetType(e.target.value)}>{ASSET_TYPES.map(t => <option key={t} value={t}>{t.replace('_', ' ')}</option>)}</select>
            <input className="form-input" style={{ width: 100 }} type="number" step="0.01" placeholder="Value" value={assetValue} onChange={e => setAssetValue(e.target.value)} required />
            <button className="btn btn-primary btn-sm"><Plus size={14} /></button>
          </form>
          {snap?.assets.map(a => (
            <div key={a.id} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 0', borderBottom: '1px solid var(--hairline)' }}>
              <span className="badge badge-green" style={{ fontSize: 10 }}>{a.asset_type}</span>
              <span style={{ flex: 1, fontSize: 13 }}>{a.name}</span>
              <span style={{ fontWeight: 600, fontSize: 13, color: 'var(--income)' }}>{formatINR(a.value)}</span>
              <button className="btn btn-ghost btn-sm" style={{ color: 'var(--expense)' }} onClick={() => delAsset(a.id)}><Trash2 size={13} /></button>
            </div>
          ))}
          {(snap?.assets.length ?? 0) === 0 && <p className="text-muted" style={{ fontSize: 12 }}>No assets yet. Add investments, property, or valuables to track your net worth.</p>}
        </div>

        <div className="card">
          <h2 style={{ fontSize: 15, fontWeight: 600, marginBottom: 12 }}>Liabilities</h2>
          <form onSubmit={addLiab} style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 12 }}>
            <input className="form-input" style={{ flex: 1, minWidth: 100 }} placeholder="Name" value={liabName} onChange={e => setLiabName(e.target.value)} required />
            <select className="form-input" style={{ width: 'auto' }} value={liabType} onChange={e => setLiabType(e.target.value)}>{LIABILITY_TYPES.map(t => <option key={t} value={t}>{t.replace('_', ' ')}</option>)}</select>
            <input className="form-input" style={{ width: 100 }} type="number" step="0.01" placeholder="Value" value={liabValue} onChange={e => setLiabValue(e.target.value)} required />
            <button className="btn btn-primary btn-sm"><Plus size={14} /></button>
          </form>
          {snap?.liabilities.map(l => (
            <div key={l.id} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 0', borderBottom: '1px solid var(--hairline)' }}>
              <span className="badge badge-red" style={{ fontSize: 10 }}>{l.liability_type}</span>
              <span style={{ flex: 1, fontSize: 13 }}>{l.name}</span>
              <span style={{ fontWeight: 600, fontSize: 13, color: 'var(--expense)' }}>{formatINR(l.value)}</span>
              <button className="btn btn-ghost btn-sm" style={{ color: 'var(--expense)' }} onClick={() => delLiab(l.id)}><Trash2 size={13} /></button>
            </div>
          ))}
          {(snap?.liabilities.length ?? 0) === 0 && <p className="text-muted" style={{ fontSize: 12 }}>No liabilities yet. Add loans, credit card debt, or other obligations.</p>}
        </div>
      </div>
    </div>
  )
}
