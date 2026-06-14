import { useEffect, useState } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, Tag } from 'lucide-react'
import { EmptyState } from '../components/EmptyState'

interface Category { id: string; name: string; txn_type: string; color: string; description: string }

export function CategoriesPage() {
  const [categories, setCategories] = useState<Category[]>([])
  const [name, setName] = useState('')
  const [type, setType] = useState('expense')
  const [color, setColor] = useState('#8479F2')
  const [desc, setDesc] = useState('')
  const [loading, setLoading] = useState(false)

  const load = () => api.get<Category[]>('/categories').then(r => setCategories(r.data)).catch(() => {})
  useEffect(() => { load() }, [])

  const add = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim()) return
    setLoading(true)
    try {
      await api.post('/categories', { name, txn_type: type, color, description: desc })
      setName(''); setDesc('')
      await load()
    } catch {} finally { setLoading(false) }
  }

  const del = async (id: string) => {
    try { await api.delete(`/categories/${id}`); await load() } catch {}
  }

  return (
    <div>
      <h1 className="page-title" style={{ marginBottom: 4 }}>Categories</h1>
      <p className="text-muted" style={{ marginBottom: 20 }}>Manage transaction categories</p>

      <div className="card" style={{ marginBottom: 20 }}>
        <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
          <input className="form-input" style={{ flex: 1, minWidth: 140 }} placeholder="Category name" value={name} onChange={e => setName(e.target.value)} required />
          <select className="form-input" style={{ width: 'auto' }} value={type} onChange={e => setType(e.target.value)}>
            <option value="expense">Expense</option>
            <option value="income">Income</option>
            <option value="investment">Investment</option>
          </select>
          <input type="color" className="form-input" style={{ width: 44, padding: 4 }} value={color} onChange={e => setColor(e.target.value)} />
          <input className="form-input" style={{ flex: 1, minWidth: 140 }} placeholder="Description (optional)" value={desc} onChange={e => setDesc(e.target.value)} />
          <button className="btn btn-primary" disabled={loading}><Plus size={15} /> Add</button>
        </form>
      </div>

      <div className="card" style={{ padding: 0 }}>
        {categories.length === 0 && <EmptyState icon="🏷️" title="No categories yet" description="Create categories to organize your income, expenses, and investments." action={{ label: 'Create a category', onClick: () => document.querySelector<HTMLInputElement>('input[placeholder*="Category name"]')?.focus() }} />}
        {categories.map(c => (
          <div key={c.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderBottom: '1px solid var(--hairline)' }}>
            <div className="stat-icon" style={{ background: `${c.color}20`, color: c.color }}><Tag size={15} /></div>
            <div style={{ flex: 1 }}>
              <div style={{ fontWeight: 600, fontSize: 13.5, color: 'var(--text)' }}>{c.name}</div>
              <div className="text-muted" style={{ fontSize: 12 }}>{c.txn_type}{c.description ? ` — ${c.description}` : ''}</div>
            </div>
            <button className="btn btn-ghost btn-sm" style={{ color: 'var(--expense)' }} onClick={() => del(c.id)}><Trash2 size={14} /></button>
          </div>
        ))}
      </div>
    </div>
  )
}
