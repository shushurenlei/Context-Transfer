import { invoke } from '@tauri-apps/api/core'

export interface ProjectEntry {
  path: string
  dir_name: string
  has_sessions: boolean
}

export interface Session {
  session_id: string
  modified: number
  size: number
}

export interface ContextInfo {
  project_path: string
  session_id: string
  messages: Message[]
  cwd: string
  git_branch: string | null
}

export interface Message {
  role: 'user' | 'assistant'
  content: string
}

export interface MigrateResult {
  success: boolean
  message: string
  filepath?: string
}

export interface CleanupResult {
  cleaned: boolean
  message: string
}

export interface ExportResult {
  success: boolean
  filepath: string
}

export type Direction = 'claude-to-codex' | 'codex-to-claude'

const USE_MOCK = false

const realApi = {
  detectProject: () => invoke<string>('detect_project'),

  listProjects: () => invoke<ProjectEntry[]>('list_projects'),

  listSessions: (projectPath: string, direction: Direction = 'claude-to-codex') =>
    invoke<Session[]>('list_sessions', { projectPath, direction }),

  extractContext: (
    projectPath: string,
    sessionId?: string,
    _maxTurns: number | null = null,
    direction: Direction = 'claude-to-codex',
  ) =>
    invoke<ContextInfo>('extract_context', {
      request: {
        projectPath,
        sessionId: sessionId || null,
        maxTurns: null,
        direction,
      },
    }),

  migrate: (
    projectPath: string,
    mode: 'prompt' | 'agents-md' | 'auto',
    _options?: {
      sessionId?: string
      model?: string
      maxTurns?: number | null
      maxLength?: number
      maxTotalLength?: number | null
      direction?: Direction
    },
  ) =>
    invoke<MigrateResult>('migrate', {
      request: {
        projectPath,
        mode,
        sessionId: _options?.sessionId || null,
        model: _options?.model || null,
        maxTurns: _options?.maxTurns ?? null,
        maxLength: _options?.maxLength || 2000,
        maxTotalLength: _options?.maxTotalLength ?? null,
        direction: _options?.direction || 'claude-to-codex',
      },
    }),

  copyPrompt: (
    projectPath: string,
    sessionId?: string,
    maxTurns: number | null = null,
    direction: Direction = 'claude-to-codex',
    maxTotalLength?: number | null,
  ) =>
    invoke<{ success: boolean; prompt: string }>('copy_prompt', {
      request: {
        projectPath,
        sessionId: sessionId || null,
        maxTurns,
        direction,
        maxTotalLength: maxTotalLength ?? null,
      },
    }),

  cleanup: (projectPath: string, direction: Direction = 'claude-to-codex') =>
    invoke<CleanupResult>('cleanup', { request: { projectPath, direction } }),

  exportContext: (
    projectPath: string,
    sessionId?: string,
    maxTurns: number | null = null,
    maxLength = 2000,
    direction: Direction = 'claude-to-codex',
  ) =>
    invoke<ExportResult>('export_context', {
      request: {
        projectPath,
        sessionId: sessionId || null,
        maxTurns,
        maxLength,
        direction,
      },
    }),
}

// --- mock (USE_MOCK = true 时) ---
const MOCK_PROJECTS: ProjectEntry[] = [
  { path: '/Users/you/projects/context-hub', dir_name: 'context-hub', has_sessions: true },
  { path: '/Users/you/projects/agent-workbench', dir_name: 'agent-workbench', has_sessions: true },
  { path: '/Users/you/projects/notes-demo', dir_name: 'notes-demo', has_sessions: false },
]

const MOCK_SESSIONS: Session[] = [
  { session_id: 'session-0f3a8c1e-context-bridge', modified: 1737360000, size: 84912 },
  { session_id: 'session-0f3a8b9f-shared-cache', modified: 1737352800, size: 62240 },
  { session_id: 'session-0f3a87b2-ui-tuning', modified: 1737349200, size: 48702 },
]

function delay(ms = 120) { return new Promise((r) => setTimeout(r, ms)) }

function mockMessages(direction: Direction, sessionId: string): Message[] {
  const target = direction === 'claude-to-codex' ? 'Codex' : 'Claude Code'
  return [
    { role: 'user', content: `我想把当前会话带到 ${target} 里继续写。` },
    { role: 'assistant', content: '建议先抽取最近消息、当前目录、分支和关键决策。' },
    { role: 'user', content: '需要保留迁移路径、关键约束、未完成任务。' },
    { role: 'assistant', content: '已记录：上下文压缩 + 迁移后续接。' },
    { role: 'user', content: `↩ session: ${sessionId}` },
  ]
}

function mockContext(projectPath: string, sessionId: string, direction: Direction): ContextInfo {
  return {
    project_path: projectPath || '/Users/you/projects/context-hub',
    session_id: sessionId || MOCK_SESSIONS[0].session_id,
    messages: mockMessages(direction, sessionId || MOCK_SESSIONS[0].session_id),
    cwd: projectPath || '/Users/you/projects/context-hub',
    git_branch: direction === 'claude-to-codex' ? 'feature/context-transfer' : 'main',
  }
}

const mockApi = {
  detectProject: async () => { await delay(80); return MOCK_PROJECTS[0].path },
  listProjects: async () => { await delay(90); return MOCK_PROJECTS },
  listSessions: async (projectPath: string, direction: Direction = 'claude-to-codex') => {
    await delay(120)
    if (projectPath?.includes('notes-demo')) return []
    if (direction === 'codex-to-claude' && !projectPath) return MOCK_SESSIONS.slice(0, 2)
    return MOCK_SESSIONS
  },
  extractContext: async (projectPath: string, sessionId?: string, _maxTurns?: number | null, direction?: Direction) => {
    await delay(140)
    return mockContext(projectPath, sessionId || MOCK_SESSIONS[0].session_id, direction || 'claude-to-codex')
  },
  migrate: async (projectPath: string, mode: string, _options?: any) => {
    await delay(260)
    return { success: true, message: `已在预览模式下模拟 ${mode} 迁移。`, filepath: `${projectPath || '/mock'}/.context-transfer/${mode}.md` }
  },
  copyPrompt: async () => {
    await delay(160)
    return { success: true, prompt: 'Mock prompt copied' }
  },
  cleanup: async (_projectPath: string, direction: Direction = 'claude-to-codex') => {
    await delay(100)
    return { cleaned: true, message: `已清理 ${direction === 'claude-to-codex' ? 'Claude Code' : 'Codex'} 的迁移痕迹（预览模式）。` }
  },
  exportContext: async (projectPath: string) => {
    await delay(140)
    return { success: true, filepath: `${projectPath || '/mock'}/context-export.md` }
  },
}

export const api = USE_MOCK ? mockApi : realApi
