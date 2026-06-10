import { useState, useMemo } from 'react'
import { api, Direction } from '../api'

interface MigratePanelProps {
  projectPath: string
  sessionId?: string
  direction: Direction
  onComplete: () => void
}

type MigrateMode = 'prompt' | 'agents-md' | 'auto'

function getModeOptions(direction: Direction) {
  const target = direction === 'codex-to-claude' ? 'Claude Code' : 'Codex'
  const mdFile = direction === 'codex-to-claude' ? 'CLAUDE.md' : 'AGENTS.md'
  return [
    {
      mode: 'prompt' as MigrateMode,
      label: 'Prompt 模式',
      desc: `生成文本并复制到剪贴板，在 ${target} 中粘贴`,
      icon: '📋',
    },
    {
      mode: 'agents-md' as MigrateMode,
      label: `${mdFile} 模式`,
      desc: `写入项目 ${mdFile}，${target} 启动自动读取`,
      icon: '📝',
    },
    {
      mode: 'auto' as MigrateMode,
      label: '一键自动模式',
      desc: `写入 ${mdFile} 并启动 ${target}`,
      icon: '🚀',
    },
  ]
}

export default function MigratePanel({
  projectPath,
  sessionId,
  direction,
  onComplete: _onComplete,
}: MigratePanelProps) {
  const modeOptions = useMemo(() => getModeOptions(direction), [direction])
  const [selectedMode, setSelectedMode] = useState<MigrateMode>('prompt')
  const [maxTurns, setMaxTurns] = useState<number | null>(null)
  const [maxLength, setMaxLength] = useState(2000)
  const [model, setModel] = useState('')
  const [migrating, setMigrating] = useState(false)
  const [result, setResult] = useState<{ success: boolean; message: string } | null>(null)

  const isClaudeToCodex = direction === 'claude-to-codex'
  const targetName = isClaudeToCodex ? 'Codex' : 'Claude Code'

  const handleMigrate = async () => {
    setMigrating(true)
    setResult(null)
    try {
      if (selectedMode === 'prompt') {
        const res = await api.copyPrompt(projectPath, sessionId, maxTurns, direction)
        setResult({ success: res.success, message: `📋 上下文已复制到剪贴板，在 ${targetName} 中粘贴即可` })
      } else {
        const res = await api.migrate(projectPath, selectedMode, {
          sessionId,
          model: model || undefined,
          maxTurns,
          maxLength,
          direction,
        })
        setResult({ success: res.success, message: res.message })
      }
    } catch (e: any) {
      setResult({ success: false, message: e.message })
    } finally {
      setMigrating(false)
    }
  }

  const handleCleanup = async () => {
    try {
      const res = await api.cleanup(projectPath, direction)
      setResult({ success: res.cleaned, message: res.message })
    } catch (e: any) {
      setResult({ success: false, message: e.message })
    }
  }

  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl overflow-hidden">
      <div className="px-5 py-4 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
        <h3 className="text-sm font-semibold text-[var(--text-primary)]">迁移到 {targetName}</h3>
        <p className="text-xs text-[var(--text-secondary)] mt-1">
          选择注入方式，将上下文迁移到 {targetName}
        </p>
      </div>

      <div className="p-5 space-y-5">
        {/* 注入方式选择 */}
        <div className="grid grid-cols-3 gap-3">
          {modeOptions.map(opt => (
            <button
              key={opt.mode}
              onClick={() => setSelectedMode(opt.mode)}
              className={`p-4 rounded-lg border text-left transition-all ${
                selectedMode === opt.mode
                  ? 'border-[var(--accent)] bg-[var(--accent)]/10'
                  : 'border-[var(--border)] bg-[var(--bg-primary)] hover:border-[var(--accent)]/50'
              }`}
            >
              <div className="text-2xl mb-2">{opt.icon}</div>
              <div className="text-sm font-medium text-[var(--text-primary)]">{opt.label}</div>
              <div className="text-xs text-[var(--text-secondary)] mt-1">{opt.desc}</div>
            </button>
          ))}
        </div>

        {/* 参数设置 */}
        <div className="grid grid-cols-3 gap-3">
          <div>
            <label className="text-xs text-[var(--text-secondary)] block mb-1">最大轮次</label>
            <input
              type="number"
              value={maxTurns ?? ''}
              onChange={e => setMaxTurns(e.target.value ? Number(e.target.value) : null)}
              placeholder="不限"
              className="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
          <div>
            <label className="text-xs text-[var(--text-secondary)] block mb-1">单条最大长度</label>
            <input
              type="number"
              value={maxLength}
              onChange={e => setMaxLength(Number(e.target.value))}
              className="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
            />
          </div>
          {isClaudeToCodex && (
            <div>
              <label className="text-xs text-[var(--text-secondary)] block mb-1">Codex 模型（可选）</label>
              <input
                type="text"
                value={model}
                onChange={e => setModel(e.target.value)}
                placeholder="默认使用配置"
                className="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
              />
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex items-center gap-3">
          <button
            onClick={handleMigrate}
            disabled={migrating}
            className="flex-1 py-3 rounded-lg bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white font-medium transition-colors disabled:opacity-50"
          >
            {migrating ? (
              <span className="flex items-center justify-center gap-2">
                <span className="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full" />
                迁移中...
              </span>
            ) : (
              `🔄 开始迁移（${modeOptions.find(o => o.mode === selectedMode)?.label}）`
            )}
          </button>
          <button
            onClick={handleCleanup}
            className="px-4 py-3 rounded-lg border border-[var(--border)] hover:bg-[var(--bg-secondary)] text-[var(--text-secondary)] text-sm transition-colors"
          >
            清理迁移
          </button>
        </div>

        {/* 结果提示 */}
        {result && (
          <div className={`p-3 rounded-lg text-sm ${
            result.success
              ? 'bg-green-50 border border-green-300 text-green-700'
              : 'bg-red-50 border border-red-300 text-red-700'
          }`}>
            {result.message}
          </div>
        )}
      </div>
    </div>
  )
}
