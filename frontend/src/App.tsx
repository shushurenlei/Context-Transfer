import { useState, useEffect, useCallback } from 'react'
import { api, Session, ContextInfo } from './api'
import SessionList from './components/SessionList'
import ContextPreview from './components/ContextPreview'
import MigratePanel from './components/MigratePanel'
import Header from './components/Header'

type Step = 'select' | 'preview' | 'migrate'

export default function App() {
  const [projectPath, setProjectPath] = useState('')
  const [sessions, setSessions] = useState<Session[]>([])
  const [selectedSession, setSelectedSession] = useState<string | null>(null)
  const [context, setContext] = useState<ContextInfo | null>(null)
  const [step, setStep] = useState<Step>('select')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 自动检测项目路径
  useEffect(() => {
    api.detectProject()
      .then(setProjectPath)
      .catch(() => setProjectPath('/'))
  }, [])

  // 加载会话列表
  const loadSessions = useCallback(async (path: string) => {
    setLoading(true)
    setError(null)
    try {
      const list = await api.listSessions(path)
      setSessions(list)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (projectPath) loadSessions(projectPath)
  }, [projectPath, loadSessions])

  // 选择会话并提取上下文
  const handleSelectSession = async (sessionId: string) => {
    setSelectedSession(sessionId)
    setLoading(true)
    setError(null)
    try {
      const ctx = await api.extractContext(projectPath, sessionId)
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
    loadSessions(projectPath)
  }

  return (
    <div className="min-h-screen bg-[var(--bg-primary)] flex flex-col">
      <Header
        projectPath={projectPath}
        onProjectPathChange={(p) => { setProjectPath(p); loadSessions(p); }}
      />

      <main className="flex-1 max-w-6xl mx-auto w-full px-6 py-6">
        {error && (
          <div className="mb-4 p-3 rounded-lg bg-red-900/30 border border-red-500/50 text-red-300 text-sm">
            {error}
          </div>
        )}

        {step === 'select' && (
          <SessionList
            sessions={sessions}
            loading={loading}
            projectPath={projectPath}
            onSelect={handleSelectSession}
            onUseLatest={handleUseLatest}
            onRefresh={() => loadSessions(projectPath)}
            onProjectPathChange={(p) => { setProjectPath(p); loadSessions(p); }}
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
              onComplete={handleMigrateDone}
            />
          </div>
        )}
      </main>
    </div>
  )
}
