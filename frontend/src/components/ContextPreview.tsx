import { useMemo } from 'react'
import type { ContextInfo, Message } from '../api'

interface Props {
  context: ContextInfo
  sourceLabel: string
  targetLabel: string
}

interface Segment { kind: 'text' | 'code'; content: string }

function parseSegments(raw: string): Segment[] {
  const segments: Segment[] = []
  const parts = raw.split(/(```[\s\S]*?```)/g)
  for (const part of parts) {
    if (!part) continue
    if (part.startsWith('```')) {
      const inner = part.replace(/^```[\w]*\n?|```$/g, '').trim()
      if (inner) segments.push({ kind: 'code', content: inner })
    } else {
      const trimmed = part.trim()
      if (trimmed) segments.push({ kind: 'text', content: trimmed })
    }
  }
  return segments
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ background: 'var(--bg)', borderRadius: 8, padding: '12px 14px', textAlign: 'center' }}>
      <div style={{ fontSize: 18, fontWeight: 700, color: 'var(--text)', lineHeight: 1.2 }}>{value}</div>
      <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 3, textTransform: 'uppercase', letterSpacing: '0.04em' }}>{label}</div>
    </div>
  )
}

function Bubble({ msg }: { msg: Message }) {
  const segments = useMemo(() => parseSegments(msg.content), [msg.content])
  const isUser = msg.role === 'user'

  return (
    <div style={{ display: 'flex', justifyContent: isUser ? 'flex-end' : 'flex-start', marginBottom: 12 }}>
      <div style={{
        maxWidth: '85%', borderRadius: 12,
        padding: '10px 14px',
        boxShadow: '0 1px 2px rgba(16,23,43,0.04)',
        ...(isUser
          ? { background: 'var(--accent-soft)', borderBottomRightRadius: 4 }
          : { background: 'var(--raised)', border: '1px solid var(--border)', borderBottomLeftRadius: 4 }),
      }}>
        <div style={{ fontSize: 10, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--muted)', marginBottom: 4 }}>
          {isUser ? '你' : 'Assistant'}
        </div>
        <div>
          {segments.map((seg, i) =>
            seg.kind === 'code' ? (
              <pre key={i} style={{
                fontSize: 12, lineHeight: 1.5, fontFamily: '"JetBrains Mono", "Fira Code", monospace',
                background: 'var(--bg)', padding: '10px 12px', borderRadius: 6, margin: '6px 0 0',
                whiteSpace: 'pre-wrap', wordBreak: 'break-word', color: 'var(--text)',
                border: '1px solid var(--border)',
              }}><code>{seg.content}</code></pre>
            ) : (
              <p key={i} style={{ fontSize: 13, lineHeight: 1.55, color: 'var(--text)', margin: 0 }}>{seg.content}</p>
            ),
          )}
        </div>
      </div>
    </div>
  )
}

export default function ContextPreview({ context, sourceLabel, targetLabel }: Props) {
  const { messages, project_path: pp, cwd, git_branch: branch } = context

  const stats = useMemo(() => {
    const userMsgs = messages.filter(m => m.role === 'user').length
    const asstMsgs = messages.filter(m => m.role === 'assistant').length
    const codeBlocks = messages.reduce((acc, m) => {
      const matches = m.content.match(/```/g)
      return acc + (matches ? Math.floor(matches.length / 2) : 0)
    }, 0)
    return { total: messages.length, user: userMsgs, assistant: asstMsgs, code: codeBlocks }
  }, [messages])

  return (
    <section style={{ display: 'flex', flexDirection: 'column', background: 'var(--surface)', overflow: 'hidden', height: '100%' }}>
      {/* Panel head */}
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '14px 16px', flexShrink: 0,
        borderBottom: '1px solid var(--border)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontSize: 13, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--muted)' }}>上下文预览</span>
        </div>
        <span style={{ fontSize: 11, fontWeight: 600, padding: '2px 8px', borderRadius: 10, background: 'var(--accent-soft)', color: 'var(--accent)' }}>
          {sourceLabel} → {targetLabel}
        </span>
      </div>

      {/* Meta bar */}
      <div style={{ padding: '10px 16px', display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: 8, borderBottom: '1px solid var(--border)', fontSize: 11, color: 'var(--muted)', fontFamily: '"JetBrains Mono", monospace' }}>
        <span>{pp}</span>
        {cwd && cwd !== pp && <><span>|</span><span>{cwd}</span></>}
        {branch && <span style={{ marginLeft: 'auto', padding: '2px 8px', borderRadius: 10, border: '1px solid var(--border)', background: 'var(--raised)', fontSize: 10 }}>{branch}</span>}
      </div>

      {/* Stats + Bubbles */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '12px 16px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 20, marginTop: 4 }}>
          <Stat label="消息总数" value={String(stats.total)} />
          <Stat label="用户提问" value={String(stats.user)} />
          <Stat label="助手回复" value={String(stats.assistant)} />
          <Stat label="代码片段" value={String(stats.code)} />
        </div>

        <div style={{
          fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em',
          color: 'var(--muted)', marginBottom: 12, paddingBottom: 6,
          borderBottom: '1px solid var(--border)',
        }}>
          对话片段
        </div>

        {messages.map((msg, i) => <Bubble key={i} msg={msg} />)}
      </div>
    </section>
  )
}
