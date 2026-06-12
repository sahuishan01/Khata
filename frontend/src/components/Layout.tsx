import { NavLink, useNavigate } from 'react-router-dom'
import { LayoutDashboard, Receipt, MessageSquare, LogOut, Shield, KeyRound, Landmark, Tags, Wallet, PiggyBank, PlusCircle, Settings } from 'lucide-react'
import { useAuth } from '../store/auth'
import { useEffect } from 'react'

const NAV = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/transactions', label: 'Transactions', icon: Receipt },
  { to: '/upload', label: 'Add Data', icon: PlusCircle },
  { to: '/chat', label: 'Ask Claude', icon: MessageSquare },
  { to: '/accounts', label: 'Accounts', icon: Landmark },
  { to: '/rules', label: 'Rules', icon: Tags },
  { to: '/budgets', label: 'Budgets', icon: PiggyBank },
  { to: '/portfolio', label: 'Portfolio', icon: Wallet },
  { to: '/profile', label: 'Settings', icon: Settings },
]

export function Layout({ children }: { children: React.ReactNode }) {
  const logout = useAuth(s => s.logout)
  const user = useAuth(s => s.user)
  const fetchMe = useAuth(s => s.fetchMe)
  const navigate = useNavigate()

  useEffect(() => { fetchMe() }, [fetchMe])

  const doLogout = () => { logout(); navigate('/login') }

  return (
    <div className="app-shell">
      {/* Desktop sidebar */}
      <aside className="sidebar">
        <div className="sidebar-brand">
          <div className="sidebar-logo">₹</div>
          <span className="sidebar-brand-name">Khata</span>
        </div>
        <nav className="sidebar-nav">
          {NAV.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={({ isActive }) => `sidebar-nav-item${isActive ? ' active' : ''}`}
            >
              <Icon size={16} />
              {label}
            </NavLink>
          ))}
          {user?.role === 'admin' && (
            <NavLink
              to="/admin/users"
              className={({ isActive }) => `sidebar-nav-item${isActive ? ' active' : ''}`}
            >
              <Shield size={16} />
              Manage Users
            </NavLink>
          )}
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
            <span style={{ fontWeight: 700, fontSize: 15, color: 'var(--text-heading)' }}>Khata</span>
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
          {NAV.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={({ isActive }) => `bottom-nav-item${isActive ? ' active' : ''}`}
            >
              <Icon size={21} />
              {label}
            </NavLink>
          ))}
          {user?.role === 'admin' && (
            <NavLink
              to="/admin/users"
              className={({ isActive }) => `bottom-nav-item${isActive ? ' active' : ''}`}
            >
              <Shield size={21} />
              Users
            </NavLink>
          )}
          <NavLink
            to="/reset-password"
            className={({ isActive }) => `bottom-nav-item${isActive ? ' active' : ''}`}
          >
            <KeyRound size={21} />
            Password
          </NavLink>
        </nav>
      </div>
    </div>
  )
}
