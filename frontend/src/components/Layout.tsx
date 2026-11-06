import { NavLink, useNavigate, useLocation } from 'react-router-dom'
import { LayoutDashboard, Receipt, PlusCircle, Grid3X3, Settings, LogOut, KeyRound } from 'lucide-react'
import { useAuth } from '../store/auth'

// DESIGN.md §4.5 — same five tabs and labels on web and Android.
const PRIMARY_NAV = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/transactions', label: 'Transactions', icon: Receipt },
  { to: '/upload', label: 'Add', icon: PlusCircle },
  { to: '/more', label: 'More', icon: Grid3X3 },
  { to: '/profile', label: 'Settings', icon: Settings },
]

// Pages reached from the More hub keep "More" highlighted.
const MORE_ROUTES = ['/more', '/analytics', '/chat', '/accounts', '/rules', '/budgets', '/portfolio',
  '/categories', '/admin', '/goals', '/subscriptions', '/reports', '/detail']

export function Wordmark({ compact = false }: { compact?: boolean }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: compact ? 10 : 12 }}>
      <div className="sidebar-logo" aria-hidden style={compact ? { width: 28, height: 28, fontSize: 12 } : undefined}>₹</div>
      <span className="sidebar-brand-name" style={compact ? { fontSize: 18 } : undefined}>Khata</span>
    </div>
  )
}

export function Layout({ children }: { children: React.ReactNode }) {
  const logout = useAuth(s => s.logout)
  const navigate = useNavigate()
  const location = useLocation()

  const doLogout = () => { logout(); navigate('/login') }

  const isActive = (to: string) => {
    if (to === '/') return location.pathname === '/'
    if (to === '/more') return MORE_ROUTES.some(r => location.pathname.startsWith(r))
    return location.pathname.startsWith(to)
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="sidebar-brand"><Wordmark /></div>
        <nav className="sidebar-nav" aria-label="Main">
          {PRIMARY_NAV.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={() => `sidebar-nav-item${isActive(to) ? ' active' : ''}`}
            >
              <Icon size={18} strokeWidth={1.75} />
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-footer">
          <button className="btn btn-ghost btn-full" style={{ justifyContent: 'flex-start', gap: 12, minHeight: 40 }} onClick={() => navigate('/reset-password')}>
            <KeyRound size={16} strokeWidth={1.75} />
            Change password
          </button>
          <button className="btn btn-ghost btn-full" style={{ justifyContent: 'flex-start', gap: 12, minHeight: 40 }} onClick={doLogout}>
            <LogOut size={16} strokeWidth={1.75} />
            Log out
          </button>
        </div>
      </aside>

      <div className="main-content">
        <header className="mobile-topbar">
          <Wordmark compact />
          <button className="btn btn-ghost btn-sm" style={{ minWidth: 44, minHeight: 44 }} onClick={doLogout} aria-label="Log out" title="Log out">
            <LogOut size={18} />
          </button>
        </header>

        <main className="page-content page-enter" key={location.pathname}>
          {children}
        </main>

        <nav className="bottom-nav" aria-label="Main">
          {PRIMARY_NAV.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={() => `bottom-nav-item${isActive(to) ? ' active' : ''}`}
            >
              <Icon size={20} strokeWidth={1.75} />
              <span>{label}</span>
            </NavLink>
          ))}
        </nav>
      </div>
    </div>
  )
}
