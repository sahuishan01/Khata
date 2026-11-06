import { useNavigate } from 'react-router-dom'
import { MessageSquare, Landmark, Tags, PiggyBank, Wallet, Shield, Tag, BarChart3, Calendar, Target, FileText } from 'lucide-react'
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
    { label: 'Analytics', route: '/analytics', icon: <BarChart3 size={18} /> },
    { label: 'Ask Claude', route: '/chat', icon: <MessageSquare size={18} /> },
    { label: 'Subscriptions', route: '/subscriptions', icon: <Calendar size={18} /> },
    { label: 'Savings Goals', route: '/goals', icon: <Target size={18} /> },
    { label: 'Reports & Export', route: '/reports', icon: <FileText size={18} /> },
    { label: 'Accounts', route: '/accounts', icon: <Landmark size={18} /> },
    { label: 'Rules', route: '/rules', icon: <Tags size={18} /> },
    { label: 'Budgets', route: '/budgets', icon: <PiggyBank size={18} /> },
    { label: 'Portfolio', route: '/portfolio', icon: <Wallet size={18} /> },
    { label: 'Categories', route: '/categories', icon: <Tag size={18} /> },
  ]

  if (user?.role === 'admin') {
    items.push({ label: 'Users', route: '/admin/users', icon: <Shield size={18} /> })
  }

  return (
    <div>
      <h1 className="page-title" style={{ marginBottom: 24 }}>More</h1>
      <div className="more-grid">
        {items.map(item => (
          <button
            key={item.route}
            className="more-card"
            onClick={() => navigate(item.route)}
          >
            <span className="more-card-icon">{item.icon}</span>
            <span className="more-card-label">{item.label}</span>
          </button>
        ))}
      </div>
    </div>
  )
}
