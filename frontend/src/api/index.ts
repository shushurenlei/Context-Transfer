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

export const api = {
  detectProject: () => invoke<string>('detect_project'),

  listProjects: () => invoke<ProjectEntry[]>('list_projects'),

  listSessions: (projectPath: string, direction: Direction = 'claude-to-codex') =>
    invoke<Session[]>('list_sessions', { projectPath, direction }),

  extractContext: (
    projectPath: string,
    sessionId?: string,
    maxTurns: number | null = null,
    direction: Direction = 'claude-to-codex',
  ) =>
    invoke<ContextInfo>('extract_context', {
      request: {
        projectPath,
        sessionId: sessionId || null,
        maxTurns,
        direction,
      },
    }),

  migrate: (
    projectPath: string,
    mode: 'prompt' | 'agents-md' | 'auto',
    options?: {
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
        sessionId: options?.sessionId || null,
        model: options?.model || null,
        maxTurns: options?.maxTurns ?? null,
        maxLength: options?.maxLength || 2000,
        maxTotalLength: options?.maxTotalLength ?? null,
        direction: options?.direction || 'claude-to-codex',
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
    invoke<CleanupResult>('cleanup', {
      request: { projectPath, direction },
    }),

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
