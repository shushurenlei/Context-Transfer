//! Tauri Commands - 暴露给前端的 API

use crate::claud_extractor::{self, ContextInfo, Session, ProjectEntry};
use crate::codex_extractor;
use crate::codex_injector;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateResult {
    pub success: bool,
    pub message: String,
    pub filepath: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub cleaned: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub success: bool,
    pub filepath: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractRequest {
    pub project_path: String,
    pub session_id: Option<String>,
    pub max_turns: Option<usize>,
    #[serde(default)]
    pub direction: String, // "claude-to-codex" (default) or "codex-to-claude"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateRequest {
    pub project_path: String,
    pub mode: String,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub max_turns: Option<usize>,
    pub max_length: Option<usize>,
    pub max_total_length: Option<usize>,
    #[serde(default)]
    pub direction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyPromptRequest {
    pub project_path: String,
    pub session_id: Option<String>,
    pub max_turns: Option<usize>,
    pub max_total_length: Option<usize>,
    #[serde(default)]
    pub direction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupRequest {
    pub project_path: String,
    #[serde(default)]
    pub direction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub project_path: String,
    pub session_id: Option<String>,
    pub max_turns: Option<usize>,
    pub max_length: Option<usize>,
    pub max_total_length: Option<usize>,
    #[serde(default)]
    pub direction: String,
}

/// 检测当前工作目录
#[tauri::command]
pub fn detect_project() -> String {
    env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "/".to_string())
}

/// 列出会话（支持方向）
#[tauri::command]
pub fn list_sessions(project_path: String, direction: Option<String>) -> Result<Vec<Session>, String> {
    let dir = direction.unwrap_or_default();
    if dir == "codex-to-claude" {
        codex_extractor::list_sessions()
    } else {
        claud_extractor::list_sessions(&project_path)
    }
}

/// 提取上下文（支持方向）
#[tauri::command]
pub fn extract_context(request: ExtractRequest) -> Result<ContextInfo, String> {
    if request.direction == "codex-to-claude" {
        codex_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )
    } else {
        claud_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )
    }
}

/// 执行迁移
#[tauri::command]
pub fn migrate(request: MigrateRequest) -> Result<MigrateResult, String> {
    let dir = request.direction.as_str();

    let context = if dir == "codex-to-claude" {
        codex_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )?
    } else {
        claud_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )?
    };

    let result = codex_injector::do_migrate(
        &context,
        &request.project_path,
        &request.mode,
        dir,
        request.model.as_deref(),
        request.max_length.unwrap_or(2000),
        request.max_total_length,
    )?;

    Ok(MigrateResult {
        success: result.success,
        message: result.message,
        filepath: result.filepath,
    })
}

/// 复制 prompt 到剪贴板
#[tauri::command]
pub fn copy_prompt(request: CopyPromptRequest) -> Result<serde_json::Value, String> {
    let dir = request.direction.as_str();

    let context = if dir == "codex-to-claude" {
        codex_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )?
    } else {
        claud_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )?
    };

    let prompt = codex_injector::copy_prompt(&context, 800, request.max_total_length)?;

    Ok(serde_json::json!({
        "success": true,
        "prompt": prompt
    }))
}

/// 清理迁移内容
#[tauri::command]
pub fn cleanup(request: CleanupRequest) -> Result<CleanupResult, String> {
    let dir = request.direction.as_str();
    let (cleaned, msg) = if dir == "codex-to-claude" {
        let c = codex_injector::cleanup_claude_md(&request.project_path)?;
        (c, if c { "已清理 CLAUDE.md" } else { "CLAUDE.md 无需清理" })
    } else {
        let c = codex_injector::cleanup_agents_md(&request.project_path)?;
        (c, if c { "已清理 AGENTS.md" } else { "AGENTS.md 无需清理" })
    };

    Ok(CleanupResult {
        cleaned,
        message: msg.to_string(),
    })
}

/// 导出上下文
#[tauri::command]
pub fn export_context(request: ExportRequest) -> Result<ExportResult, String> {
    let context = if request.direction == "codex-to-claude" {
        codex_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )?
    } else {
        claud_extractor::extract_context(
            &request.project_path,
            request.session_id.as_deref(),
            request.max_turns,
        )?
    };

    let md = crate::context_formatter::format_as_markdown(
        &context,
        request.max_length.unwrap_or(2000),
        request.max_total_length,
    );

    let filepath = std::path::Path::new(&request.project_path)
        .join("context_export.md");
    fs::write(&filepath, &md)
        .map_err(|e| format!("写入导出文件失败: {}", e))?;

    Ok(ExportResult {
        success: true,
        filepath: filepath.to_string_lossy().to_string(),
    })
}

/// 列出所有已知的 Claude Code 项目
#[tauri::command]
pub fn list_projects() -> Result<Vec<ProjectEntry>, String> {
    claud_extractor::list_projects()
}

use std::fs;
