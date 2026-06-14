import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { ProtectedRoute } from './components/ProtectedRoute'
import { Layout } from './components/Layout'
import { ChatPage } from './pages/ChatPage'
import { DashboardPage } from './pages/DashboardPage'
import { LoginPage } from './pages/LoginPage'
import { SetupPage } from './pages/SetupPage'
import { TransactionsPage } from './pages/TransactionsPage'
import { UploadPage } from './pages/UploadPage'
import { ProfilePage } from './pages/ProfilePage'
import { ResetPasswordPage } from './pages/ResetPasswordPage'
import { AdminUsersPage } from './pages/AdminUsersPage'
import { AccountsPage } from './pages/AccountsPage'
import { RulesPage } from './pages/RulesPage'
import { BudgetsPage } from './pages/BudgetsPage'
import { PortfolioPage } from './pages/PortfolioPage'
import { MorePage } from './pages/MorePage'
import { CategoriesPage } from './pages/CategoriesPage'
import { AnalyticsPage } from './pages/AnalyticsPage'

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
        <Route path="/upload"       element={<AppLayout><UploadPage /></AppLayout>} />
        <Route path="/chat"         element={<AppLayout><ChatPage /></AppLayout>} />
        <Route path="/profile"      element={<AppLayout><ProfilePage /></AppLayout>} />
        <Route path="/accounts"     element={<AppLayout><AccountsPage /></AppLayout>} />
        <Route path="/rules"        element={<AppLayout><RulesPage /></AppLayout>} />
        <Route path="/budgets"      element={<AppLayout><BudgetsPage /></AppLayout>} />
        <Route path="/portfolio"    element={<AppLayout><PortfolioPage /></AppLayout>} />
        <Route path="/categories"   element={<AppLayout><CategoriesPage /></AppLayout>} />
        <Route path="/analytics"    element={<AppLayout><AnalyticsPage /></AppLayout>} />
        <Route path="/more"         element={<AppLayout><MorePage /></AppLayout>} />
        <Route path="/admin/users"  element={<AppLayout><AdminUsersPage /></AppLayout>} />
        <Route path="*"             element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}
