import { useState } from 'react'
import type { Session } from '../api'

interface SessionListProps {
  sessions: Session[]
  loading: boolean
  projectPath: string
  onSelect: (sessionId: string) => void
  onUseLatest: () => void
  onRefresh: () => void
  onProjectPathChange: (path: string) => void
}

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleString('zh-CN', {
    month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit',
  })
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}

export default function SessionList({
  sessions, loading, projectPath,
  onSelect, onUseLatest, onRefresh,
}: SessionListProps) {
  const [hoveredId, setHoveredId] = useState<string | null>(null)

  return (
    <div className="space-y-4">
      {/* 操作栏 */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-base font-medium text-[var(--text-primary)]">
            Claude Code 会话
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            选择一个会话来提取上下文
          </p>
        </div>
        <div className="flex gap-2">
          {sessions.length > 0 && (
            <button
              onClick={onUseLatest}
              className="px-4 py-2 rounded-lg bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white text-sm font-medium transition-colors"
            >
              使用最新会话
            </button>
          )}
          <button
            onClick={onRefresh}
            disabled={loading}
            className="px-3 py-2 rounded-lg border border-[var(--border)] hover:bg-[var(--bg-secondary)] text-[var(--text-secondary)] text-sm transition-colors disabled:opacity-50"
          >
            {loading ? '加载中...' : '刷新'}
          </button>
        </div>
      </div>

      {/* 会话列表 */}
      {loading ? (
        <div className="flex items-center justify-center py-20 text-[var(--text-secondary)]">
          <div className="animate-spin w-6 h-6 border-2 border-[var(--accent)] border-t-transparent rounded-full mr-3" />
          加载会话列表...
        </div>
      ) : sessions.length === 0 ? (
        <div className="text-center py-20">
          <p className="text-[var(--text-secondary)] text-lg mb-2">未找到 Claude Code 会话</p>
          <p className="text-[var(--text-secondary)] text-sm">
            请确认项目路径正确，且在 Claude Code 中使用过该项目
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {sessions.map((s, i) => (
            <button
              key={s.session_id}
              onClick={() => onSelect(s.session_id)}
              onMouseEnter={() => setHoveredId(s.session_id)}
              onMouseLeave={() => setHoveredId(null)}
              className={`w-full text-left p-4 rounded-lg border transition-all ${
                hoveredId === s.session_id
                  ? 'border-[var(--accent)] bg-[var(--accent)]/5'
                  : 'border-[var(--border)] bg-[var(--bg-card)]'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className="text-xs bg-[var(--bg-primary)] text-[var(--text-secondary)] px-2 py-1 rounded">
                    #{i + 1}
                  </span>
                  <span className="text-sm font-mono text-[var(--text-primary)]">
                    {s.session_id.slice(0, 20)}...
                  </span>
                </div>
                <div className="flex items-center gap-4 text-xs text-[var(--text-secondary)]">
                  <span>{formatTime(s.modified)}</span>
                  <span>{formatSize(s.size)}</span>
                  {hoveredId === s.session_id && (
                    <span className="text-[var(--accent)]">→ 点击提取</span>
                  )}
                </div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
