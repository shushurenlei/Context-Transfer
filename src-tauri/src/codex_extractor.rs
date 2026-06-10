//! 从 Codex CLI 会话文件中提取对话上下文

use crate::claud_extractor::{ContextInfo, Message, Session};
use serde::Deserialize;
use std::fs;

/// Codex 历史记录条目
#[derive(Debug, Deserialize)]
struct CodexEntry {
    session_id: String,
    ts: f64,
    text: String,
}

/// Codex history.jsonl 路径
fn codex_history_path() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("无法获取 HOME 目录")
        .join(".codex")
        .join("history.jsonl")
}

/// 按时间倒序列出所有 Codex 会话
pub fn list_sessions() -> Result<Vec<Session>, String> {
    let path = codex_history_path();
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("读取 Codex 历史失败: {}", e))?;

    let mut sessions: std::collections::HashMap<String, (f64, u64, usize)> =
        std::collections::HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<CodexEntry>(line) {
            let e = sessions
                .entry(entry.session_id.clone())
                .or_insert((entry.ts, 0, 0));
            e.0 = e.0.max(entry.ts); // latest timestamp
            e.1 += entry.text.len() as u64; // total size
            e.2 += 1; // message count
        }
    }

    let mut result: Vec<Session> = sessions
        .into_iter()
        .map(|(id, (ts, size, _count))| Session {
            session_id: id,
            modified: ts,
            size,
        })
        .collect();

    result.sort_by(|a, b| b.modified.partial_cmp(&a.modified).unwrap());
    Ok(result)
}

/// 提取指定 Codex 会话的上下文
pub fn extract_context(
    project_path: &str,
    session_id: Option<&str>,
    max_turns: Option<usize>,
) -> Result<ContextInfo, String> {
    let sessions = list_sessions()?;
    if sessions.is_empty() {
        return Err("未找到 Codex 会话记录".to_string());
    }

    let target = if let Some(sid) = session_id {
        sessions
            .iter()
            .find(|s| s.session_id == sid)
            .ok_or_else(|| format!("Codex 会话 ID '{}' 不存在", sid))?
            .session_id
            .clone()
    } else {
        sessions[0].session_id.clone()
    };

    let path = codex_history_path();
    let content = fs::read_to_string(&path).map_err(|e| format!("读取 Codex 历史失败: {}", e))?;

    let mut messages: Vec<Message> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<CodexEntry>(line) {
            if entry.session_id == target {
                messages.push(Message {
                    role: "user".to_string(),
                    content: entry.text,
                });
            }
        }
    }

    // 合并连续 user 消息
    let mut merged: Vec<Message> = Vec::new();
    for msg in messages {
        if let Some(last) = merged.last_mut() {
            if last.role == msg.role {
                last.content.push_str(&format!("\n---\n{}", msg.content));
                continue;
            }
        }
        merged.push(msg);
    }

    let limited = match max_turns {
        Some(limit) => merged.into_iter().take(limit).collect(),
        None => merged,
    };

    Ok(ContextInfo {
        project_path: project_path.to_string(),
        session_id: target,
        messages: limited,
        cwd: project_path.to_string(),
        git_branch: None,
    })
}
