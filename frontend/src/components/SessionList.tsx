import { useState } from 'react'
import type { Direction, ProjectEntry, Session } from '../api'

interface Props {
  sessions: Session[]
  loading: boolean
  direction: Direction
  knownProjects: ProjectEntry[]
  selectedId: string | null
  onSelect: (sessionId: string) => void
  onUseLatest: () => void
  onRefresh: () => void
  onProjectPathChange: (path: string) => void
}

function formatTime(ts: number) {
  return new Date(ts * 1000).toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export default function SessionList({ sessions, loading, direction, knownProjects, selectedId, onSelect, onRefresh, onProjectPathChange }: Props) {
  const sourceLabel = direction === 'codex-to-claude' ? 'Codex' : 'Claude Code'
  const [hoveredId, setHoveredId] = useState<string | null>(null)
  const [projectInput, setProjectInput] = useState('')

  return (
    <section
      style={{
        display: 'flex', flexDirection: 'column',
        background: 'var(--surface)', overflow: 'hidden',
        height: '100%',
      }}
    >
      {/* Panel head */}
      <div
        style={{
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          padding: '14px 16px', flexShrink: 0,
          borderBottom: '1px solid var(--border)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontSize: 13, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--muted)' }}>源会话</span>
          <span style={{ fontSize: 11, fontWeight: 600, padding: '2px 8px', borderRadius: 10, background: 'var(--bg)', color: 'var(--muted)' }}>{sessions.length}</span>
        </div>
        <div style={{ display: 'flex', gap: 6 }}>
          <button
            type="button" onClick={onRefresh} disabled={loading}
            style={{ padding: '4px 10px', borderRadius: 16, border: '1px solid var(--border)', background: 'var(--raised)', fontSize: 11, fontWeight: 600, color: 'var(--muted)', cursor: 'pointer', fontFamily: 'inherit' }}
          >
            {loading ? '刷新中…' : '刷新'}
          </button>
        </div>
      </div>

      {/* Project path input */}
      <div style={{ padding: '10px 12px', borderBottom: '1px solid var(--border)', display: 'flex', flexDirection: 'column', gap: 6 }}>
        <div style={{ display: 'flex', gap: 6 }}>
          <input
            type="text" value={projectInput}
            onChange={e => setProjectInput(e.target.value)}
            placeholder="输入项目路径（或从下方点选）"
            style={{ flex: 1, padding: '6px 12px', borderRadius: 16, border: '1px solid var(--border)', background: 'var(--bg)', fontSize: 12, color: 'var(--text)', outline: 'none', fontFamily: 'inherit' }}
            onKeyDown={e => { if (e.key === 'Enter') { const p = projectInput.trim(); if (p) onProjectPathChange(p) } }}
          />
          <button
            type="button" onClick={() => { const p = projectInput.trim(); if (p) onProjectPathChange(p) }}
            style={{ padding: '6px 12px', borderRadius: 16, border: '1px solid var(--border)', background: 'var(--raised)', fontSize: 12, fontWeight: 600, color: 'var(--muted)', cursor: 'pointer', fontFamily: 'inherit' }}
          >
            确定
          </button>
        </div>
        {knownProjects.length > 0 && (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
            {knownProjects.map(p => (
              <button
                key={p.path} type="button"
                onClick={() => onProjectPathChange(p.path)}
                style={{ padding: '3px 10px', borderRadius: 12, border: '1px solid var(--border)', background: 'var(--bg)', fontSize: 10, fontWeight: 500, color: 'var(--muted)', cursor: 'pointer', fontFamily: 'inherit', whiteSpace: 'nowrap' }}
              >
                {p.dir_name}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Scrollable list */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '12px' }}>
        {loading ? (
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 32, color: 'var(--muted)', fontSize: 13 }}>
            加载中…
          </div>
        ) : sessions.length === 0 ? (
          <div style={{ padding: 24, textAlign: 'center', color: 'var(--muted)', fontSize: 12, lineHeight: 1.6 }}>
            暂无会话，输入项目路径后刷新
          </div>
        ) : (
          sessions.map((s) => {
            const active = selectedId === s.session_id || hoveredId === s.session_id
            return (
              <button
                key={s.session_id} type="button"
                onClick={() => onSelect(s.session_id)}
                onMouseEnter={() => setHoveredId(s.session_id)}
                onMouseLeave={() => setHoveredId(null)}
                style={{
                  display: 'block', width: '100%', textAlign: 'left',
                  padding: '12px 14px', marginBottom: 8,
                  borderRadius: 12,
                  border: active ? '1px solid var(--accent)' : '1px solid var(--border)',
                  background: active ? 'var(--raised)' : 'var(--surface)',
                  boxShadow: active ? '0 0 0 1px var(--accent), 0 2px 8px rgba(224,82,45,0.1)' : 'none',
                  cursor: 'pointer', fontFamily: 'inherit',
                  transition: 'all 0.15s ease', lineHeight: 1.4,
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 5 }}>
                  <span style={{ fontSize: 10, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--muted)' }}>{sourceLabel}</span>
                  {selectedId === s.session_id && <span style={{ fontSize: 8, color: 'var(--success)' }}>●</span>}
                  <span style={{ fontSize: 11, color: 'var(--muted)', marginLeft: 'auto' }}>{s.size ? formatSize(s.size) : '—'}</span>
                </div>
                <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text)', marginBottom: 3 }}>{s.session_id.slice(0, 28)}…</div>
                <div style={{ fontSize: 11, color: 'var(--muted)', marginBottom: 4 }}>{formatTime(s.modified)}</div>
              </button>
            )
          })
        )}
      </div>
    </section>
  )
}
