import { useEffect, useRef, useState } from 'react'
import { Send, ChevronDown, ChevronUp, Bot } from 'lucide-react'
import { api } from '../api/client'

interface Message {
  id: string; role: string; content: string; sql_used?: string
}

const SUGGESTIONS = [
  'How much did I spend this month?',
  'How much did I spend last month?',
  'What are my top expenses?',
  'What is my savings rate?',
  'Show monthly breakdown',
  'What is my net balance?',
  'What is my largest expense?',
  'How many transactions do I have?',
]

export function ChatPage() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput]       = useState('')
  const [loading, setLoading]   = useState(false)
  const [showSql, setShowSql]   = useState<string | null>(null)
  const bottomRef = useRef<HTMLDivElement>(null)
  const inputRef  = useRef<HTMLInputElement>(null)

  useEffect(() => {
    api.get<Message[]>('/chat/history').then(r => setMessages(r.data))
  }, [])

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const send = async (text?: string) => {
    const question = (text ?? input).trim()
    if (!question || loading) return
    setInput('')
    setMessages(m => [...m, { id: Date.now().toString(), role: 'user', content: question }])
    setLoading(true)
    try {
      const { data } = await api.post<{ answer: string; sql_used: string }>('/chat/ask', { question })
      setMessages(m => [...m, {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: data.answer,
        sql_used: data.sql_used,
      }])
    } catch (e: unknown) {
      const err = e as { response?: { data?: { error?: string } } }
      setMessages(m => [...m, {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `Error: ${err.response?.data?.error ?? 'Failed'}`,
      }])
    } finally {
      setLoading(false)
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }

  const isEmpty = messages.length === 0

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100svh - var(--nav-h) - 48px)' }}>
      <div className="flex items-center justify-between mb-4">
        <h1 className="page-title">Ask Claude</h1>
        <span className="badge badge-purple">
          <Bot size={11} style={{ marginRight: 3 }} /> AI
        </span>
      </div>

      {/* Messages */}
      <div style={{ flex: 1, overflowY: 'auto', paddingRight: 4 }}>
        {isEmpty && (
          <div style={{ textAlign: 'center', padding: '32px 0 16px' }}>
            <div style={{ fontSize: 36, marginBottom: 12 }}>💬</div>
            <p style={{ color: 'var(--text-2)', fontSize: 14, marginBottom: 20 }}>
              Ask anything about your transactions
            </p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, justifyContent: 'center' }}>
              {SUGGESTIONS.map(s => (
                <button
                  key={s}
                  className="btn btn-secondary btn-sm"
                  onClick={() => send(s)}
                >
                  {s}
                </button>
              ))}
            </div>
          </div>
        )}

        {messages.map(m => (
          <div
            key={m.id}
            style={{
              marginBottom: 14,
              display: 'flex',
              flexDirection: 'column',
              alignItems: m.role === 'user' ? 'flex-end' : 'flex-start',
            }}
          >
            <div className={`chat-bubble ${m.role === 'user' ? 'chat-bubble-user' : 'chat-bubble-bot'}`}>
              {m.content}
            </div>
            {m.sql_used && (
              <div style={{ marginTop: 4 }}>
                <button
                  onClick={() => setShowSql(showSql === m.id ? null : m.id)}
                  className="btn btn-ghost btn-sm"
                  style={{ fontSize: 11, color: 'var(--text-2)', gap: 3 }}
                >
                  {showSql === m.id ? <ChevronUp size={11} /> : <ChevronDown size={11} />}
                  {showSql === m.id ? 'hide SQL' : 'show SQL'}
                </button>
                {showSql === m.id && (
                  <pre style={{
                    fontSize: 12,
                    background: 'var(--surface-2)',
                    border: '1px solid var(--border)',
                    padding: '8px 12px',
                    borderRadius: 'var(--r-md)',
                    overflowX: 'auto',
                    margin: '4px 0 0',
                    color: 'var(--text-heading)',
                    lineHeight: 1.5,
                  }}>
                    {m.sql_used}
                  </pre>
                )}
              </div>
            )}
          </div>
        ))}

        {loading && (
          <div style={{ display: 'flex', alignItems: 'flex-start', gap: 8, marginBottom: 14 }}>
            <div className="chat-bubble chat-bubble-bot" style={{ padding: '10px 16px' }}>
              <span style={{ display: 'inline-flex', gap: 4, alignItems: 'center' }}>
                <span className="dot-pulse" style={{ animationDelay: '0ms' }}>·</span>
                <span className="dot-pulse" style={{ animationDelay: '150ms' }}>·</span>
                <span className="dot-pulse" style={{ animationDelay: '300ms' }}>·</span>
              </span>
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      {/* Input bar */}
      <div className="chat-input-row">
        <input
          ref={inputRef}
          className="form-input"
          style={{ flex: 1 }}
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={e => e.key === 'Enter' && !e.shiftKey && send()}
          placeholder="How much did I spend last month?"
          disabled={loading}
        />
        <button
          className="btn btn-primary"
          onClick={() => send()}
          disabled={loading || !input.trim()}
          style={{ flexShrink: 0 }}
        >
          <Send size={15} />
        </button>
      </div>

      <style>{`
        .dot-pulse {
          font-size: 20px;
          line-height: 1;
          animation: pulse 1s infinite;
          color: var(--text-2);
        }
        @keyframes pulse {
          0%, 80%, 100% { opacity: 0.2; }
          40% { opacity: 1; }
        }
        @media (max-width: 768px) {
          /* Chat fills screen minus mobile bottom nav */
        }
      `}</style>
    </div>
  )
}
