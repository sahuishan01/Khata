import { NavLink, useNavigate, useLocation } from 'react-router-dom'
import { LayoutDashboard, Receipt, PlusCircle, Grid3X3, Settings, LogOut, Shield, KeyRound } from 'lucide-react'
import { useAuth } from '../store/auth'

const PRIMARY_NAV = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/transactions', label: 'Transactions', icon: Receipt },
  { to: '/upload', label: 'Add', icon: PlusCircle },
  { to: '/more', label: 'More', icon: Grid3X3 },
  { to: '/profile', label: 'Settings', icon: Settings },
]

export function Layout({ children }: { children: React.ReactNode }) {
  const logout = useAuth(s => s.logout)
  const user = useAuth(s => s.user)
  const navigate = useNavigate()
  const location = useLocation()

  const doLogout = () => { logout(); navigate('/login') }

  const isActive = (to: string) => {
    if (to === '/') return location.pathname === '/'
    if (to === '/more') return location.pathname.startsWith('/more')
    return location.pathname.startsWith(to)
  }

  return (
    <div className="app-shell">
      {/* Desktop sidebar */}
      <aside className="sidebar">
        <div className="sidebar-brand">
          <div className="sidebar-logo">₹</div>
          <span className="sidebar-brand-name">Khata</span>
        </div>
        <nav className="sidebar-nav">
          {PRIMARY_NAV.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={({ isActive: act }) => `sidebar-nav-item${act || isActive(to) ? ' active' : ''}`}
            >
              <Icon size={16} />
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-footer">
          <button
            className="btn btn-ghost btn-full"
            style={{ justifyContent: 'flex-start', gap: 10 }}
            onClick={() => navigate('/reset-password')}
          >
            <KeyRound size={15} />
            Reset Password
          </button>
          <button
            className="btn btn-ghost btn-full"
            style={{ justifyContent: 'flex-start', gap: 10 }}
            onClick={doLogout}
          >
            <LogOut size={15} />
            Logout
          </button>
        </div>
      </aside>

      {/* Content area */}
      <div className="main-content">
        {/* Mobile top bar */}
        <header className="mobile-topbar">
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <div className="sidebar-logo" style={{ width: 28, height: 28, fontSize: 13 }}>₹</div>
            <span style={{ fontWeight: 700, fontSize: 15 }}>Khata</span>
          </div>
          <button className="btn btn-ghost btn-sm" onClick={doLogout} title="Logout">
            <LogOut size={16} />
          </button>
        </header>

        <div className="page-content">
          {children}
        </div>

        {/* Mobile bottom nav */}
        <nav className="bottom-nav">
          {PRIMARY_NAV.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={({ isActive: act }) => `bottom-nav-item${act || isActive(to) ? ' active' : ''}`}
            >
              <Icon size={21} />
              <span>{label}</span>
            </NavLink>
          ))}
        </nav>
      </div>
    </div>
  )
}
