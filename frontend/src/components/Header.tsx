import type { Direction } from '../api'

interface HeaderProps {
  projectPath: string
  direction: Direction
  onDirectionChange: (dir: Direction) => void
  onProjectPathChange: (path: string) => void
}

const LABELS: Record<Direction, { title: string; sub: string; source: string; target: string }> = {
  'claude-to-codex': {
    title: 'Context Reset',
    sub: 'Claude Code → Codex 上下文迁移',
    source: 'Claude Code',
    target: 'Codex',
  },
  'codex-to-claude': {
    title: 'Context Reset',
    sub: 'Codex → Claude Code 上下文迁移',
    source: 'Codex',
    target: 'Claude Code',
  },
}

export default function Header({
  projectPath,
  direction,
  onDirectionChange,
  onProjectPathChange,
}: HeaderProps) {
  const info = LABELS[direction]
  const isClaudeToCodex = direction === 'claude-to-codex'

  return (
    <header className="bg-[var(--bg-secondary)] border-b border-[var(--border)] px-6 py-4">
      <div className="max-w-6xl mx-auto flex items-center justify-between gap-4">
        <div className="flex items-center gap-3 shrink-0">
          <div className="w-9 h-9 rounded-lg bg-[var(--accent)] flex items-center justify-center text-white font-bold text-lg">
            🔄
          </div>
          <div>
            <h1 className="text-lg font-semibold text-[var(--text-primary)]">{info.title}</h1>
            <p className="text-xs text-[var(--text-secondary)]">{info.sub}</p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {/* 方向切换 */}
          <div className="flex items-center rounded-lg border border-[var(--border)] overflow-hidden">
            <button
              onClick={() => onDirectionChange('claude-to-codex')}
              className={`px-3 py-1.5 text-xs font-medium transition-colors ${
                isClaudeToCodex
                  ? 'bg-[var(--accent)] text-white'
                  : 'bg-[var(--bg-card)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              }`}
            >
              Claude → Codex
            </button>
            <button
              onClick={() => onDirectionChange('codex-to-claude')}
              className={`px-3 py-1.5 text-xs font-medium transition-colors ${
                !isClaudeToCodex
                  ? 'bg-[var(--accent)] text-white'
                  : 'bg-[var(--bg-card)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              }`}
            >
              Codex → Claude
            </button>
          </div>

          {/* 项目路径 */}
          {isClaudeToCodex && (
            <div className="flex items-center gap-2">
              <label className="text-xs text-[var(--text-secondary)]">项目路径</label>
              <input
                type="text"
                value={projectPath}
                onChange={(e) => onProjectPathChange(e.target.value)}
                className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] w-72 focus:outline-none focus:border-[var(--accent)]"
                placeholder="/path/to/project"
              />
            </div>
          )}
        </div>
      </div>
    </header>
  )
}
