import type { Direction } from '../api'

interface HeaderProps {
  projectPath: string
  direction: Direction
  onDirectionChange: (dir: Direction) => void
  onProjectPathChange: (path: string) => void
}

export default function Header({
  projectPath,
  direction,
  onDirectionChange,
  onProjectPathChange,
}: HeaderProps) {
  const isClaudeToCodex = direction === 'claude-to-codex'

  return (
    <header className="bg-[var(--bg-secondary)] border-b border-[var(--border)] px-6 py-4">
      <div className="max-w-6xl mx-auto flex items-center gap-4">
        {/* 左侧：Logo + 标题 */}
        <div className="flex items-center gap-3 shrink-0">
          <div className="w-9 h-9 rounded-lg bg-[var(--accent)] flex items-center justify-center text-white font-bold text-lg">
            🔄
          </div>
          <div>
            <h1 className="text-lg font-semibold text-[var(--text-primary)]">Context Transfer</h1>
          </div>
        </div>

        {/* 中间：方向切换 */}
        <div className="flex-1 flex justify-center">
          <div className="flex items-center rounded-lg border border-[var(--border)] overflow-hidden">
            <button
              onClick={() => onDirectionChange('claude-to-codex')}
              className={`px-4 py-1.5 text-sm font-medium transition-colors ${
                isClaudeToCodex
                  ? 'bg-[var(--accent)] text-white'
                  : 'bg-[var(--bg-card)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              }`}
            >
              Claude → Codex
            </button>
            <button
              onClick={() => onDirectionChange('codex-to-claude')}
              className={`px-4 py-1.5 text-sm font-medium transition-colors ${
                !isClaudeToCodex
                  ? 'bg-[var(--accent)] text-white'
                  : 'bg-[var(--bg-card)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              }`}
            >
              Codex → Claude
            </button>
          </div>
        </div>

        {/* 右侧：项目路径（仅 Claude → Codex 时显示） */}
        {isClaudeToCodex && (
          <div className="flex items-center gap-2 shrink-0">
            <label className="text-xs text-[var(--text-secondary)] whitespace-nowrap">项目路径</label>
            <input
              type="text"
              value={projectPath}
              onChange={(e) => onProjectPathChange(e.target.value)}
              className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] w-60 focus:outline-none focus:border-[var(--accent)]"
              placeholder="点击下方已知项目或手动输入"
            />
          </div>
        )}
      </div>
    </header>
  )
}
