import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { ProtectedRoute } from './components/ProtectedRoute'
import { Layout } from './components/Layout'
import { ChatPage } from './pages/ChatPage'
import { DashboardPage } from './pages/DashboardPage'
import { LoginPage } from './pages/LoginPage'
import { SetupPage } from './pages/SetupPage'
import { TransactionsPage } from './pages/TransactionsPage'
import { ResetPasswordPage } from './pages/ResetPasswordPage'
import { AdminUsersPage } from './pages/AdminUsersPage'

function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <ProtectedRoute>
      <Layout>{children}</Layout>
    </ProtectedRoute>
  )
}

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/setup"        element={<SetupPage />} />
        <Route path="/login"        element={<LoginPage />} />
        <Route path="/reset-password" element={<ResetPasswordPage />} />
        <Route path="/"             element={<AppLayout><DashboardPage /></AppLayout>} />
        <Route path="/transactions" element={<AppLayout><TransactionsPage /></AppLayout>} />
        <Route path="/chat"         element={<AppLayout><ChatPage /></AppLayout>} />
        <Route path="/admin/users"  element={<AppLayout><AdminUsersPage /></AppLayout>} />
        <Route path="*"             element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}
