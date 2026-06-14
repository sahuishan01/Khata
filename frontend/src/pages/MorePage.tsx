import { useNavigate } from 'react-router-dom'
import { MessageSquare, Landmark, Tags, PiggyBank, Wallet, Shield, Tag } from 'lucide-react'
import { useAuth } from '../store/auth'

interface MoreItem {
  label: string
  route: string
  icon: React.ReactNode
}

export function MorePage() {
  const navigate = useNavigate()
  const user = useAuth(s => s.user)

  const items: MoreItem[] = [
    { label: 'Ask Claude', route: '/chat', icon: <MessageSquare size={22} /> },
    { label: 'Accounts', route: '/accounts', icon: <Landmark size={22} /> },
    { label: 'Rules', route: '/rules', icon: <Tags size={22} /> },
    { label: 'Budgets', route: '/budgets', icon: <PiggyBank size={22} /> },
    { label: 'Portfolio', route: '/portfolio', icon: <Wallet size={22} /> },
    { label: 'Categories', route: '/categories', icon: <Tag size={22} /> },
  ]

  if (user?.role === 'admin') {
    items.push({ label: 'Users', route: '/admin/users', icon: <Shield size={22} /> })
  }

  return (
    <div>
      <h1 className="page-title" style={{ marginBottom: 4 }}>More</h1>
      <p className="text-muted" style={{ marginBottom: 24 }}>All features</p>
      <div className="more-grid">
        {items.map(item => (
          <div
            key={item.route}
            className="more-card"
            onClick={() => navigate(item.route)}
          >
            <div className="more-card-icon">{item.icon}</div>
            <span className="more-card-label">{item.label}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
