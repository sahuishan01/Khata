import { useEffect, useState } from 'react'
import { api } from '../api/client'
import { Plus, Trash2, Tag } from 'lucide-react'
import { Screen, Card, CardBody, ListRow, ListRowText, Field, Select, Button, EmptyState } from '../components/shared'

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
    <Screen title="Categories">
      <div style={{ marginBottom: 20 }}>
        <Card>
          <CardBody>
            <form onSubmit={add} style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              <Field style={{ flex: 1, minWidth: 140 }} placeholder="Category name" value={name} onChange={e => setName(e.target.value)} required />
              <Select value={type} onChange={e => setType(e.target.value)} style={{ width: 'auto' }}>
                <option value="expense">Expense</option>
                <option value="income">Income</option>
                <option value="investment">Investment</option>
              </Select>
              <input type="color" style={{ width: 44, padding: 4, background: 'var(--surface-2)', border: '1px solid var(--hairline)', borderRadius: 8, cursor: 'pointer' }} value={color} onChange={e => setColor(e.target.value)} />
              <Field style={{ flex: 1, minWidth: 140 }} placeholder="Description (optional)" value={desc} onChange={e => setDesc(e.target.value)} />
              <Button disabled={loading}><Plus size={15} /> Add</Button>
            </form>
          </CardBody>
        </Card>
      </div>

      <Card>
        {categories.length === 0 && <EmptyState icon="🏷️" title="No categories yet" description="Create categories to organize your income, expenses, and investments." action={{ label: 'Create a category', onClick: () => document.querySelector<HTMLInputElement>('input[placeholder*="Category name"]')?.focus() }} />}
        {categories.map(c => (
          <ListRow
            key={c.id}
            leading={<div style={{ width: 32, height: 32, borderRadius: 8, background: `${c.color}20`, color: c.color, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}><Tag size={15} /></div>}
            trailing={<Button variant="ghost" size="sm" style={{ color: 'var(--expense)' }} onClick={() => del(c.id)}><Trash2 size={14} /></Button>}
          >
            <ListRowText primary={c.name} secondary={c.txn_type + (c.description ? ` — ${c.description}` : '')} />
          </ListRow>
        ))}
      </Card>
    </Screen>
  )
}
