import { useState, useEffect, useCallback } from 'react'
import { api, Session, ContextInfo, ProjectEntry, Direction } from './api'
import SessionList from './components/SessionList'
import ContextPreview from './components/ContextPreview'
import MigratePanel from './components/MigratePanel'
import Header from './components/Header'

type Step = 'select' | 'preview' | 'migrate'

export default function App() {
  const [projectPath, setProjectPath] = useState('')
  const [direction, setDirection] = useState<Direction>('claude-to-codex')
  const [sessions, setSessions] = useState<Session[]>([])
  const [selectedSession, setSelectedSession] = useState<string | null>(null)
  const [context, setContext] = useState<ContextInfo | null>(null)
  const [step, setStep] = useState<Step>('select')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [knownProjects, setKnownProjects] = useState<ProjectEntry[]>([])

  // 加载已知项目
  useEffect(() => {
    api.listProjects()
      .then(setKnownProjects)
      .catch(() => {})
  }, [])

  // Codex 方向时自动加载会话（无需项目路径）
  useEffect(() => {
    if (direction === 'codex-to-claude' && !projectPath) {
      loadSessions('', direction)
    }
  }, [direction])

  // 加载会话列表
  const loadSessions = useCallback(async (path: string, dir: Direction) => {
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
  }, [])

  useEffect(() => {
    if (projectPath && direction === 'claude-to-codex') loadSessions(projectPath, direction)
  }, [projectPath, direction, loadSessions])

  // 切换方向
  const handleDirectionChange = (dir: Direction) => {
    setDirection(dir)
    setProjectPath('')
    setSessions([])
    setStep('select')
    setContext(null)
    setSelectedSession(null)
  }

  // 选择会话并提取上下文
  const handleSelectSession = async (sessionId: string) => {
    setSelectedSession(sessionId)
    setLoading(true)
    setError(null)
    try {
      const ctx = await api.extractContext(projectPath, sessionId, null, direction)
      setContext(ctx)
      setStep('preview')
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  // 使用最新会话
  const handleUseLatest = async () => {
    if (sessions.length === 0) return
    handleSelectSession(sessions[0].session_id)
  }

  // 返回选择
  const handleBack = () => {
    setStep('select')
    setContext(null)
    setSelectedSession(null)
  }

  // 迁移完成后
  const handleMigrateDone = () => {
    setStep('select')
    setContext(null)
    setSelectedSession(null)
    loadSessions(projectPath, direction)
  }

  return (
    <div className="min-h-screen bg-[var(--bg-primary)] flex flex-col">
      <Header
        projectPath={projectPath}
        direction={direction}
        onDirectionChange={handleDirectionChange}
        onProjectPathChange={(p) => { setProjectPath(p); loadSessions(p, direction); }}
      />

      <main className="flex-1 max-w-6xl mx-auto w-full px-6 py-6">
        {error && (
          <div className="mb-4 p-3 rounded-lg bg-red-50 border border-red-300 text-red-700 text-sm">
            {error}
          </div>
        )}

        {step === 'select' && (
          <SessionList
            sessions={sessions}
            loading={loading}
            projectPath={projectPath}
            direction={direction}
            knownProjects={knownProjects}
            onSelect={handleSelectSession}
            onUseLatest={handleUseLatest}
            onRefresh={() => loadSessions(projectPath, direction)}
            onProjectPathChange={(p) => { setProjectPath(p); loadSessions(p, direction); }}
          />
        )}

        {step === 'preview' && context && (
          <div className="space-y-4">
            <button
              onClick={handleBack}
              className="text-[var(--text-secondary)] hover:text-[var(--text-primary)] text-sm flex items-center gap-1"
            >
              ← 返回会话列表
            </button>
            <ContextPreview context={context} />
            <MigratePanel
              projectPath={projectPath}
              sessionId={selectedSession || undefined}
              direction={direction}
              onComplete={handleMigrateDone}
            />
          </div>
        )}
      </main>
    </div>
  )
}
