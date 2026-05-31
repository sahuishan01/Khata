import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { ProtectedRoute } from './components/ProtectedRoute'
import { ChatPage } from './pages/ChatPage'
import { DashboardPage } from './pages/DashboardPage'
import { LoginPage } from './pages/LoginPage'
import { RegisterPage } from './pages/RegisterPage'
import { TransactionsPage } from './pages/TransactionsPage'

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login"    element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/" element={<ProtectedRoute><DashboardPage /></ProtectedRoute>} />
        <Route path="/transactions" element={<ProtectedRoute><TransactionsPage /></ProtectedRoute>} />
        <Route path="/chat"     element={<ProtectedRoute><ChatPage /></ProtectedRoute>} />
        <Route path="*"         element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}
