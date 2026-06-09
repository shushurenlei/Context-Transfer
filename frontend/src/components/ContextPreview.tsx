import type { ContextInfo, Message } from '../api'

interface ContextPreviewProps {
  context: ContextInfo
}

function MessageBubble({ msg, index }: { msg: Message; index: number }) {
  const isUser = msg.role === 'user'
  const isToolResult = msg.content.startsWith('↩')

  if (isUser && isToolResult) {
    return (
      <div className="py-1 px-3 text-xs text-[var(--text-secondary)] bg-[var(--bg-primary)] rounded border border-[var(--border)] max-h-20 overflow-hidden">
        {msg.content.slice(0, 150)}
      </div>
    )
  }

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`max-w-[80%] rounded-lg px-4 py-2.5 text-sm leading-relaxed ${
          isUser
            ? 'bg-[var(--accent)] text-white'
            : 'bg-[var(--bg-secondary)] text-[var(--text-primary)] border border-[var(--border)]'
        }`}
      >
        <div className="text-xs opacity-60 mb-1">
          {isUser ? '👤 用户' : '🤖 助手'}
        </div>
        {msg.content.length > 500
          ? msg.content.slice(0, 500) + '...'
          : msg.content
        }
      </div>
    </div>
  )
}

export default function ContextPreview({ context }: ContextPreviewProps) {
  const userMsgs = context.messages.filter(m => m.role === 'user' && !m.content.startsWith('↩'))

  return (
    <div className="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl overflow-hidden">
      {/* 摘要头 */}
      <div className="px-5 py-4 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-semibold text-[var(--text-primary)]">上下文预览</h3>
          <span className="text-xs bg-[var(--accent)]/20 text-[var(--accent)] px-2 py-0.5 rounded">
            {context.messages.length} 条消息
          </span>
        </div>
        <div className="flex gap-4 text-xs text-[var(--text-secondary)]">
          <span>📁 {context.project_path}</span>
          {context.git_branch && <span>🌿 {context.git_branch}</span>}
          <span>👤 用户提问 {userMsgs.length} 次</span>
        </div>
      </div>

      {/* 消息列表 */}
      <div className="p-4 space-y-3 max-h-[400px] overflow-y-auto">
        {context.messages.map((msg, i) => (
          <MessageBubble key={i} msg={msg} index={i} />
        ))}
      </div>
    </div>
  )
}
