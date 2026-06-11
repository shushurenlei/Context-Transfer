import type { Direction } from '../api'

interface Props {
  direction: Direction
  onDirectionChange: (dir: Direction) => void
}

export default function Header({ direction, onDirectionChange }: Props) {
  const isForward = direction === 'claude-to-codex'
  const sourceLabel = isForward ? 'Claude Code' : 'Codex'

  const pillBase: React.CSSProperties = {
    display: 'flex', alignItems: 'center', gap: 6,
    padding: '6px 16px', borderRadius: 24,
    border: 'none', cursor: 'pointer',
    fontSize: 13, fontWeight: 600,
    transition: 'all 0.2s ease',
    fontFamily: 'inherit', lineHeight: 1.4,
  }

  return (
    <header
      style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '0 24px', height: 56,
        background: 'var(--surface)',
        borderBottom: '1px solid var(--border)',
        flexShrink: 0, zIndex: 10,
      }}
    >
      {/* Logo */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--accent)', lineHeight: 1 }}>⇋</div>
        <div style={{ display: 'flex', flexDirection: 'column' }}>
          <span style={{ fontSize: 14, fontWeight: 700, lineHeight: 1.2, letterSpacing: '-0.01em' }}>Context Migrate</span>
          <span style={{ fontSize: 10, color: 'var(--muted)', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.06em' }}>alpha</span>
        </div>
      </div>

      {/* Direction Toggle Pill */}
      <div
        role="radiogroup"
        aria-label="迁移方向"
        style={{
          display: 'flex', gap: 0,
          background: 'var(--bg)', borderRadius: 28,
          border: '1px solid var(--border)', padding: 3,
        }}
      >
        <button
          role="radio"
          aria-checked={isForward}
          onClick={() => onDirectionChange('claude-to-codex')}
          style={{
            ...pillBase,
            ...(isForward
              ? { background: 'var(--raised)', boxShadow: '0 1px 3px rgba(16,23,43,0.08), 0 0 0 1px rgba(16,23,43,0.04)', color: 'var(--text)' }
              : { background: 'transparent', color: 'var(--muted)' }),
          }}
        >
          <span>Claude Code</span>
          <span style={{ fontSize: 16, fontWeight: 700, color: isForward ? 'var(--accent)' : 'var(--border)' }}>→</span>
          <span>Codex</span>
        </button>
        <button
          role="radio"
          aria-checked={!isForward}
          onClick={() => onDirectionChange('codex-to-claude')}
          style={{
            ...pillBase,
            ...(!isForward
              ? { background: 'var(--raised)', boxShadow: '0 1px 3px rgba(16,23,43,0.08), 0 0 0 1px rgba(16,23,43,0.04)', color: 'var(--text)' }
              : { background: 'transparent', color: 'var(--muted)' }),
          }}
        >
          <span>Codex</span>
          <span style={{ fontSize: 16, fontWeight: 700, color: !isForward ? 'var(--accent)' : 'var(--border)' }}>→</span>
          <span>Claude Code</span>
        </button>
      </div>

      {/* Status */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '4px 12px', borderRadius: 20, background: 'var(--bg)', fontSize: 12, color: 'var(--muted)' }}>
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--success)', flexShrink: 0 }} />
        <span style={{ whiteSpace: 'nowrap' }}>{sourceLabel} 已连接</span>
      </div>
    </header>
  )
}
