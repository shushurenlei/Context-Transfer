//! 从 Codex CLI 数据中提取会话上下文
//!
//! 数据来源：
//! - ~/.codex/state_5.sqlite → threads 表（会话列表，含 cwd、标题、时间）
//! - ~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl → 完整对话（user + assistant）

use crate::claud_extractor::{ContextInfo, Message, Session};
use rusqlite::Connection;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

fn codex_home() -> PathBuf {
    dirs::home_dir()
        .expect("无法获取 HOME 目录")
        .join(".codex")
}

/// 从 state_5.sqlite 的 threads 表列出所有 Codex 会话
pub fn list_sessions() -> Result<Vec<Session>, String> {
    let db_path = codex_home().join("state_5.sqlite");
    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("打开 Codex 数据库失败: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, updated_at, tokens_used, cwd, first_user_message
             FROM threads
             WHERE archived = 0
             ORDER BY updated_at DESC",
        )
        .map_err(|e| format!("查询 Codex 会话失败: {}", e))?;

    let sessions: Vec<Session> = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let updated_at: i64 = row.get(1)?;
            let tokens_used: i64 = row.get(2)?;
            Ok(Session {
                session_id: id,
                modified: updated_at as f64,
                size: tokens_used as u64,
            })
        })
        .map_err(|e| format!("读取 Codex 会话失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sessions)
}

/// 查找指定会话的 rollout 文件路径
fn find_rollout_path(session_id: &str) -> Option<PathBuf> {
    let sessions_dir = codex_home().join("sessions");
    if !sessions_dir.exists() {
        return None;
    }

    // 递归搜索 sessions/ 目录
    fn search_dir(dir: &std::path::Path, sid: &str) -> Option<PathBuf> {
        if !dir.is_dir() {
            return None;
        }
        let entries = fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = search_dir(&path, sid) {
                    return Some(found);
                }
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains(sid) && path.extension().and_then(|e| e.to_str()) == Some("jsonl")
                {
                    return Some(path);
                }
            }
        }
        None
    }

    search_dir(&sessions_dir, session_id)
}

/// 提取指定 Codex 会话的完整上下文（含 user + assistant 消息）
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

    // 从数据库获取 cwd
    let (cwd, _title) = get_thread_info(&target).unwrap_or_else(|_| (project_path.to_string(), String::new()));

    // 解析 rollout 文件
    let messages = match find_rollout_path(&target) {
        Some(path) => parse_rollout(&path)?,
        None => {
            // 回退到 history.jsonl
            parse_history(&target)?
        }
    };

    // 合并连续同角色消息
    let merged = merge_consecutive(messages);

    let limited = match max_turns {
        Some(limit) => {
            let total = merged.len();
            merged.into_iter().skip(total.saturating_sub(limit)).collect()
        }
        None => merged,
    };

    Ok(ContextInfo {
        project_path: cwd.clone(),
        session_id: target,
        messages: limited,
        cwd,
        git_branch: None,
    })
}

/// 从数据库获取 thread 的 cwd 和标题
fn get_thread_info(session_id: &str) -> Result<(String, String), String> {
    let db_path = codex_home().join("state_5.sqlite");
    let conn = Connection::open(&db_path).map_err(|e| format!("打开 Codex 数据库失败: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT cwd, first_user_message FROM threads WHERE id = ?1")
        .map_err(|e| format!("查询 thread 信息失败: {}", e))?;

    stmt.query_row([session_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })
    .map_err(|e| format!("读取 thread 信息失败: {}", e))
}

/// 解析 Codex rollout JSONL 文件，提取 user/assistant 消息
fn parse_rollout(path: &std::path::Path) -> Result<Vec<Message>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("读取 rollout 文件失败: {}", e))?;

    #[derive(Deserialize)]
    struct EventMsg {
        #[serde(rename = "type")]
        event_type: String,
        message: Option<String>,
        phase: Option<String>,
    }

    #[derive(Deserialize)]
    struct RolloutEntry {
        #[serde(rename = "type")]
        entry_type: String,
        payload: Option<serde_json::Value>,
    }

    let mut messages: Vec<Message> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry: RolloutEntry = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.entry_type != "event_msg" {
            continue;
        }

        let payload = match entry.payload {
            Some(p) => p,
            None => continue,
        };

        let event: EventMsg = match serde_json::from_value(payload) {
            Ok(e) => e,
            Err(_) => continue,
        };

        match event.event_type.as_str() {
            "user_message" => {
                if let Some(text) = event.message {
                    if !text.trim().is_empty() {
                        messages.push(Message {
                            role: "user".to_string(),
                            content: text.trim().to_string(),
                        });
                    }
                }
            }
            "agent_message" => {
                if let Some(text) = event.message {
                    if !text.trim().is_empty() {
                        // 跳过纯 commentary（只保留 final_answer）
                        let role = if event.phase.as_deref() == Some("final_answer") {
                            "assistant"
                        } else {
                            "assistant"
                        };
                        messages.push(Message {
                            role: role.to_string(),
                            content: text.trim().to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    Ok(messages)
}

/// 回退：从 history.jsonl 提取用户消息（无 assistant 回复）
fn parse_history(session_id: &str) -> Result<Vec<Message>, String> {
    let path = codex_home().join("history.jsonl");
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("读取 history.jsonl 失败: {}", e))?;

    #[derive(Deserialize)]
    struct HistoryEntry {
        session_id: String,
        text: String,
    }

    let messages: Vec<Message> = content
        .lines()
        .filter_map(|line| {
            let e: HistoryEntry = serde_json::from_str(line.trim()).ok()?;
            if e.session_id == session_id && !e.text.trim().is_empty() {
                Some(Message {
                    role: "user".to_string(),
                    content: e.text.trim().to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(messages)
}

/// 合并连续同角色消息
fn merge_consecutive(msgs: Vec<Message>) -> Vec<Message> {
    let mut merged: Vec<Message> = Vec::new();
    for msg in msgs {
        if let Some(last) = merged.last_mut() {
            if last.role == msg.role {
                last.content.push_str(&format!("\n{}", msg.content));
                continue;
            }
        }
        merged.push(msg);
    }
    merged
}
