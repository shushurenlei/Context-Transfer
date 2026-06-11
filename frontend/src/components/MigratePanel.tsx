import { useState } from 'react'
import { api } from '../api'
import type { Direction } from '../api'

interface Props {
  projectPath: string
  sessionId?: string
  direction: Direction
  onComplete: () => void
}

type Mode = 'auto' | 'file' | 'prompt'
type Status = 'idle' | 'running' | 'done'

interface ModeOption {
  key: Mode
  label: string
  desc: string
  action: string
}

export default function MigratePanel({ projectPath, sessionId, direction, onComplete }: Props) {
  const [mode, setMode] = useState<Mode>('auto')
  const [truncate, setTruncate] = useState(200)
  const [status, setStatus] = useState<Status>('idle')
  const [resultMsg, setResultMsg] = useState('')

  const label = direction === 'claude-to-codex'
    ? { src: 'Claude Code', tgt: 'Codex', mdFile: 'CLAUDE.md' }
    : { src: 'Codex', tgt: 'Claude Code', mdFile: 'AGENTS.md' }

  const MODES: ModeOption[] = [
    {
      key: 'auto',
      label: '一键自动',
      desc: `写入 ${label.mdFile} 并自动启动 ${label.tgt} 终端`,
      action: `写入并启动 ${label.tgt}`,
    },
    {
      key: 'file',
      label: '文件写入',
      desc: `只写入 ${label.mdFile}，${label.tgt} 启动时自动读取`,
      action: `写入 ${label.mdFile}`,
    },
    {
      key: 'prompt',
      label: '复制 Prompt',
      desc: '生成上下文文本复制到剪贴板，手动粘贴到目标工具',
      action: '复制到剪贴板',
    },
  ]

  const canMigrate = !!sessionId && status !== 'running'
  const activeMode = MODES.find(m => m.key === mode)!

  const handleMigrate = async () => {
    if (!canMigrate) return
    setStatus('running')
    try {
      if (mode === 'auto') {
        const res = await api.migrate(projectPath, 'auto', {
          sessionId, direction, maxLength: 2000,
        })
        setResultMsg(res.message)
      } else if (mode === 'file') {
        const res = await api.migrate(projectPath, 'agents-md', {
          sessionId, direction, maxLength: 2000,
        })
        setResultMsg(res.message)
      } else {
        const res = await api.copyPrompt(projectPath, sessionId, truncate, direction, null)
        setResultMsg(res.prompt ? '上下文已复制到剪贴板，直接粘贴即可' : '已复制')
      }
      setStatus('done')
      onComplete()
      setTimeout(() => setStatus('idle'), 3000)
    } catch (e: any) {
      setResultMsg(e.message || '迁移失败')
      setStatus('done')
      setTimeout(() => setStatus('idle'), 3000)
    }
  }

  return (
    <section style={{ display: 'flex', flexDirection: 'column', background: 'var(--surface)', overflow: 'hidden', height: '100%' }}>
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '14px 16px', flexShrink: 0,
        borderBottom: '1px solid var(--border)',
      }}>
        <span style={{ fontSize: 13, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--muted)' }}>迁移控制</span>
        <span style={{ fontSize: 11, fontWeight: 600, padding: '2px 8px', borderRadius: 10, background: 'var(--accent-soft)', color: 'var(--accent)' }}>
          → {label.tgt}
        </span>
      </div>

      <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column' }}>
        {/* Mode cards */}
        <div style={{ padding: '16px 16px 0' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--muted)', marginBottom: 10 }}>
            迁移方式
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }} role="radiogroup" aria-label="迁移方式">
            {MODES.map(opt => (
              <label
                key={opt.key}
                style={{
                  display: 'block', padding: '10px 12px', borderRadius: 8,
                  border: mode === opt.key ? '1px solid var(--accent)' : '1px solid var(--border)',
                  cursor: 'pointer', transition: 'all 0.15s ease',
                  background: mode === opt.key ? 'var(--raised)' : 'var(--surface)',
                  boxShadow: mode === opt.key ? '0 0 0 1px var(--accent)' : 'none',
                  position: 'relative',
                }}
              >
                <input type="radio" name="migrate-mode" checked={mode === opt.key} onChange={() => setMode(opt.key)}
                  style={{ position: 'absolute', opacity: 0, pointerEvents: 'none', width: 0, height: 0 }} />
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 2 }}>
                  <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--text)' }}>{opt.label}</span>
                  {opt.key === 'auto' && (
                    <span style={{ fontSize: 9, fontWeight: 700, padding: '1px 6px', borderRadius: 6, background: 'var(--accent)', color: '#fff', textTransform: 'uppercase' }}>推荐</span>
                  )}
                  {mode === opt.key && (
                    <span style={{ marginLeft: 'auto', width: 8, height: 8, borderRadius: '50%', background: 'var(--accent)', flexShrink: 0 }} />
                  )}
                </div>
                <div style={{ fontSize: 11, color: 'var(--muted)' }}>{opt.desc}</div>
              </label>
            ))}
          </div>
        </div>

        {/* Truncation slider (prompt mode) */}
        {mode === 'prompt' && (
          <div style={{ padding: '0 16px 16px', marginTop: 16 }}>
            <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--muted)', marginBottom: 10, display: 'flex', justifyContent: 'space-between' }}>
              <span>消息条数</span>
              <span style={{ fontWeight: 500, textTransform: 'none', letterSpacing: '0', fontSize: 12 }}>最近 {truncate} 条</span>
            </div>
            <input
              type="range" min={20} max={500} step={10} value={truncate}
              onChange={e => setTruncate(+e.target.value)}
              style={{ width: '100%', margin: '8px 0 0', accentColor: 'var(--accent)' }}
            />
          </div>
        )}

        {/* Output info */}
        <div style={{ padding: '0 16px 16px' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--muted)', marginBottom: 10 }}>
            输出
          </div>
          <div style={{
            display: 'flex', alignItems: 'center', gap: 8,
            padding: '8px 12px', borderRadius: 8,
            border: '1px solid var(--border)', background: 'var(--bg)',
          }}>
            <code style={{
              flex: 1, fontSize: 12, fontFamily: '"JetBrains Mono", "Fira Code", monospace',
              color: 'var(--text)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
            }}>
              {mode === 'prompt' ? '系统剪贴板' : `${projectPath || '~/project'}/${label.mdFile}`}
            </code>
          </div>
        </div>

        {/* Action */}
        <div style={{ padding: '12px 16px 16px', borderTop: '1px solid var(--border)', marginTop: 'auto' }}>
          {status === 'done' && (
            <div style={{
              display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
              padding: '8px 12px', borderRadius: 8, marginBottom: 12,
              background: '#e8f5e9', color: 'var(--success)', fontSize: 13, fontWeight: 600,
            }} role="status">
              <span>✓</span> {resultMsg || `已完成`}
            </div>
          )}
          <button
            type="button"
            onClick={handleMigrate}
            disabled={!canMigrate}
            style={{
              display: 'block', width: '100%', padding: '12px 0', borderRadius: 24,
              border: 'none', cursor: canMigrate ? 'pointer' : 'not-allowed',
              fontSize: 14, fontWeight: 700, fontFamily: 'inherit',
              letterSpacing: '0.01em', transition: 'all 0.15s ease',
              background: mode === 'auto'
                ? (canMigrate ? 'var(--accent)' : 'var(--border)')
                : (canMigrate ? 'var(--text)' : 'var(--border)'),
              color: canMigrate ? '#fff' : 'var(--muted)',
            }}
          >
            {status === 'idle' && activeMode.action}
            {status === 'running' && '迁移中…'}
            {status === 'done' && '完成 ✓'}
          </button>
          <p style={{ fontSize: 11, color: 'var(--muted)', textAlign: 'center', marginTop: 10, lineHeight: 1.4 }}>
            {sessionId
              ? `${label.src} → ${label.tgt}`
              : '请先在左侧选择一个源会话'}
          </p>
        </div>
      </div>
    </section>
  )
}
