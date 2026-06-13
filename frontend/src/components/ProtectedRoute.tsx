import { Navigate } from 'react-router-dom'
import { useAuth } from '../store/auth'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const user = useAuth(s => s.user)
  const loading = useAuth(s => s.loading)
  if (loading) return null
  return user ? <>{children}</> : <Navigate to="/login" replace />
}
