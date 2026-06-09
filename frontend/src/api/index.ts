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

export const api = {
  detectProject: () => invoke<string>('detect_project'),

  listProjects: () => invoke<ProjectEntry[]>('list_projects'),

  listSessions: (projectPath: string) =>
    invoke<Session[]>('list_sessions', { projectPath }),

  extractContext: (projectPath: string, sessionId?: string, maxTurns = 50) =>
    invoke<ContextInfo>('extract_context', {
      request: {
        projectPath,
        sessionId: sessionId || null,
        maxTurns,
      },
    }),

  migrate: (projectPath: string, mode: 'prompt' | 'agents-md' | 'auto', options?: {
    sessionId?: string
    model?: string
    maxTurns?: number
    maxLength?: number
  }) =>
    invoke<MigrateResult>('migrate', {
      request: {
        projectPath,
        mode,
        sessionId: options?.sessionId || null,
        model: options?.model || null,
        maxTurns: options?.maxTurns || 50,
        maxLength: options?.maxLength || 2000,
      },
    }),

  copyPrompt: (projectPath: string, sessionId?: string, maxTurns = 50) =>
    invoke<{ success: boolean; prompt: string }>('copy_prompt', {
      request: {
        projectPath,
        sessionId: sessionId || null,
        maxTurns,
      },
    }),

  cleanup: (projectPath: string) =>
    invoke<CleanupResult>('cleanup', {
      request: { projectPath },
    }),

  exportContext: (projectPath: string, sessionId?: string, maxTurns = 50, maxLength = 2000) =>
    invoke<ExportResult>('export_context', {
      request: {
        projectPath,
        sessionId: sessionId || null,
        maxTurns,
        maxLength,
      },
    }),
}
