import { useEffect, useMemo, useState } from 'react'
import { api } from './api'
import type { ContextInfo, Direction, ProjectEntry, Session } from './api'
import ContextPreview from './components/ContextPreview'
import Header from './components/Header'
import MigratePanel from './components/MigratePanel'
import SessionList from './components/SessionList'

export default function App() {
  const [projectPath, setProjectPath] = useState('')
  const [direction, setDirection] = useState<Direction>('claude-to-codex')
  const [sessions, setSessions] = useState<Session[]>([])
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [context, setContext] = useState<ContextInfo | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [knownProjects, setKnownProjects] = useState<ProjectEntry[]>([])

  const isForward = direction === 'claude-to-codex'
  const sourceLabel = isForward ? 'Claude Code' : 'Codex'
  const targetLabel = isForward ? 'Codex' : 'Claude Code'
  const visibleProjects = useMemo(() => knownProjects.filter(p => p.has_sessions), [knownProjects])

  useEffect(() => { api.listProjects().then(setKnownProjects).catch(() => {}) }, [])

  useEffect(() => {
    if (direction === 'codex-to-claude' && !projectPath) loadSessions('', direction)
  }, [direction, projectPath])

  const loadSessions = async (path: string, dir: Direction) => {
    setLoading(true)
    setError(null)
    try {
      const list = await api.listSessions(path, dir)
      setSessions(list)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (projectPath && direction === 'claude-to-codex') loadSessions(projectPath, direction)
  }, [projectPath, direction])

  const handleDirectionChange = (dir: Direction) => {
    setDirection(dir)
    setProjectPath('')
    setSessions([])
    setContext(null)
    setSelectedId(null)
    setError(null)
  }

  const handleSelectSession = async (sessionId: string) => {
    setSelectedId(sessionId)
    setLoading(true)
    setError(null)
    try {
      const ctx = await api.extractContext(projectPath, sessionId, null, direction)
      setContext(ctx)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  const handleUseLatest = () => {
    if (sessions.length === 0) return
    handleSelectSession(sessions[0].session_id)
  }

  const handleMigrateDone = () => {
    setContext(null)
    setSelectedId(null)
    loadSessions(projectPath, direction)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: 'var(--bg)', color: 'var(--text)', overflow: 'hidden', position: 'relative' }}>
      <Header direction={direction} onDirectionChange={handleDirectionChange} />

      {error && (
        <div style={{ padding: '8px 24px', background: 'var(--surface)', borderBottom: '1px solid var(--border)' }}>
          <div style={{ padding: '8px 14px', borderRadius: 8, background: 'rgba(196,61,50,0.08)', color: 'var(--error)', fontSize: 13 }}>{error}</div>
        </div>
      )}

      {/* Three-column workspace */}
      <main style={{ display: 'flex', flex: 1, overflow: 'hidden', gap: 1, background: 'var(--border)' }}>
        {/* Column 1 — Session List */}
        <div style={{ width: 320, flexShrink: 0 }}>
          <SessionList
            sessions={sessions}
            loading={loading}
            direction={direction}
            knownProjects={visibleProjects}
            selectedId={selectedId}
            onSelect={handleSelectSession}
            onUseLatest={handleUseLatest}
            onRefresh={() => loadSessions(projectPath, direction)}
            onProjectPathChange={p => { setProjectPath(p); loadSessions(p, direction) }}
          />
        </div>

        {/* Column 2 — Context Preview */}
        <div style={{ flex: 1, minWidth: 0 }}>
          {context ? (
            <ContextPreview context={context} sourceLabel={sourceLabel} targetLabel={targetLabel} />
          ) : (
            <div style={{
              display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
              height: '100%', background: 'var(--surface)', padding: 32, gap: 12,
            }}>
              <div style={{ fontSize: 32, color: 'var(--border)' }}>⇠</div>
              <div style={{ fontSize: 15, fontWeight: 600, color: 'var(--text)' }}>选择一个会话</div>
              <div style={{ fontSize: 13, color: 'var(--muted)', textAlign: 'center', maxWidth: 260 }}>
                从左侧列表选择要预览的会话上下文
              </div>
            </div>
          )}
        </div>

        {/* Column 3 — Migrate Panel */}
        <div style={{ width: 320, flexShrink: 0 }}>
          <MigratePanel
            projectPath={projectPath}
            sessionId={selectedId || undefined}
            direction={direction}
            onComplete={handleMigrateDone}
          />
        </div>
      </main>
    </div>
  )
}
