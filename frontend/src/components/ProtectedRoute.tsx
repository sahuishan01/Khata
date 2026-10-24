import { Navigate, useLocation } from 'react-router-dom'
import { useAuth } from '../store/auth'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const location = useLocation()
  const user = useAuth(s => s.user)
  const loading = useAuth(s => s.loading)
  if (loading) return null
  return user ? <>{children}</> : <Navigate to="/login" state={{ returnTo: location.pathname }} replace />
}
