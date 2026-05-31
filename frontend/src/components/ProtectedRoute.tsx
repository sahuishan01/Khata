import { Navigate } from 'react-router-dom'
import { useAuth } from '../store/auth'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const token = useAuth(s => s.token)
  return token ? <>{children}</> : <Navigate to="/login" replace />
}
