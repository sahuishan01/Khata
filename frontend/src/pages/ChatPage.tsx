import { useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../api/client'

interface Message {
  id: string; role: string; content: string; sql_used?: string
}

export function ChatPage() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const [showSql, setShowSql] = useState<string | null>(null)
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    api.get<Message[]>('/chat/history').then(r => setMessages(r.data))
  }, [])
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const send = async () => {
    if (!input.trim() || loading) return
    const question = input.trim()
    setInput('')
    setMessages(m => [...m, { id: Date.now().toString(), role: 'user', content: question }])
    setLoading(true)
    try {
      const { data } = await api.post<{ answer: string; sql_used: string }>('/chat/ask', { question })
      setMessages(m => [...m, {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: data.answer,
        sql_used: data.sql_used
      }])
    } catch (e: unknown) {
      const err = e as { response?: { data?: { error?: string } } }
      setMessages(m => [...m, {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `Error: ${err.response?.data?.error ?? 'Failed'}`
      }])
    } finally {
      setLoading(false)
    }
  }

  return (
    <div style={{ maxWidth: 720, margin: '0 auto', padding: 24, display: 'flex', flexDirection: 'column', height: '100vh', boxSizing: 'border-box' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2 style={{ margin: 0 }}>Ask Claude</h2>
        <Link to="/">← Dashboard</Link>
      </div>
      <div style={{ flex: 1, overflowY: 'auto', padding: '16px 0' }}>
        {messages.length === 0 && (
          <p style={{ color: '#999', fontStyle: 'italic' }}>
            Ask anything about your transactions — e.g. "How much did I spend on food last month?"
          </p>
        )}
        {messages.map(m => (
          <div key={m.id} style={{ marginBottom: 16, textAlign: m.role === 'user' ? 'right' : 'left' }}>
            <div style={{
              display: 'inline-block',
              background: m.role === 'user' ? '#2563eb' : '#f3f4f6',
              color: m.role === 'user' ? '#fff' : '#111',
              borderRadius: 8, padding: '8px 14px', maxWidth: '80%', textAlign: 'left'
            }}>
              {m.content}
            </div>
            {m.sql_used && (
              <div style={{ textAlign: 'left' }}>
                <button onClick={() => setShowSql(showSql === m.id ? null : m.id!)}
                  style={{ fontSize: 11, color: '#888', background: 'none', border: 'none', cursor: 'pointer', padding: '2px 0' }}>
                  {showSql === m.id ? 'hide SQL ▲' : 'show SQL ▼'}
                </button>
                {showSql === m.id && (
                  <pre style={{ fontSize: 12, background: '#f8f8f8', padding: 8, borderRadius: 4, overflowX: 'auto', margin: 0 }}>
                    {m.sql_used}
                  </pre>
                )}
              </div>
            )}
          </div>
        ))}
        {loading && <div style={{ color: '#999', fontStyle: 'italic' }}>Claude is thinking…</div>}
        <div ref={bottomRef} />
      </div>
      <div style={{ display: 'flex', gap: 8, borderTop: '1px solid #eee', paddingTop: 12 }}>
        <input value={input} onChange={e => setInput(e.target.value)}
          onKeyDown={e => e.key === 'Enter' && !e.shiftKey && send()}
          placeholder="How much did I spend last month?"
          style={{ flex: 1, padding: '10px 14px', borderRadius: 8, border: '1px solid #ddd', fontSize: 15 }} />
        <button onClick={send} disabled={loading}
          style={{ padding: '10px 20px', borderRadius: 8, background: '#2563eb', color: '#fff', border: 'none', cursor: 'pointer' }}>
          Send
        </button>
      </div>
    </div>
  )
}
