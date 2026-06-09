interface HeaderProps {
  projectPath: string
  onProjectPathChange: (path: string) => void
}

export default function Header({ projectPath, onProjectPathChange }: HeaderProps) {
  return (
    <header className="bg-[var(--bg-secondary)] border-b border-[var(--border)] px-6 py-4">
      <div className="max-w-6xl mx-auto flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-lg bg-[var(--accent)] flex items-center justify-center text-white font-bold text-lg">
            🔄
          </div>
          <div>
            <h1 className="text-lg font-semibold text-[var(--text-primary)]">Context Reset</h1>
            <p className="text-xs text-[var(--text-secondary)]">Claude Code → Codex 上下文迁移</p>
          </div>
        </div>
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
      </div>
    </header>
  )
}
