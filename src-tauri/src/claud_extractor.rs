//! 从 Claude Code 会话文件中提取对话上下文

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Claude Code home 目录
fn claude_home() -> PathBuf {
    dirs::home_dir()
        .expect("无法获取 HOME 目录")
        .join(".claude")
}

/// 将 cwd 转换为 Claude Code 项目目录名格式
/// 映射规则：/ -> -, _ -> -, 保留前导 -
fn get_project_dir(cwd: &str) -> String {
    cwd.replace('/', "-").replace('_', "-")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub modified: f64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub project_path: String,
    pub session_id: String,
    pub messages: Vec<Message>,
    pub cwd: String,
    pub git_branch: Option<String>,
}

/// 列出指定项目的所有会话，按时间倒序
pub fn list_sessions(project_path: &str) -> Result<Vec<Session>, String> {
    let project_dir = get_project_dir(project_path);
    let project_base = claude_home().join("projects").join(&project_dir);

    if !project_base.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();
    let entries = fs::read_dir(&project_base).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let metadata = fs::metadata(&path).map_err(|e| format!("读取文件元数据失败: {}", e))?;
        let session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        sessions.push(Session {
            session_id,
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            size: metadata.len(),
        });
    }

    sessions.sort_by(|a, b| b.modified.partial_cmp(&a.modified).unwrap());
    Ok(sessions)
}

/// 将单个工具调用摘要为可读文本
fn tool_call_summary(tc: &serde_json::Value) -> String {
    let name = tc.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");

    if name == "tool_result" {
        let preview = tc
            .get("result_preview")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        return preview.chars().take(80).collect();
    }

    if let Some(input) = tc.get("input") {
        if let Some(cmd) = input.get("command").and_then(|v| v.as_str()) {
            return format!("执行: {}", &cmd[..cmd.len().min(80)]);
        }
        if let Some(desc) = input.get("description").and_then(|v| v.as_str()) {
            return format!("{}: {}", name, &desc[..desc.len().min(80)]);
        }
        if let Some(p) = input.get("path").and_then(|v| v.as_str()) {
            return format!("{}: {}", name, &p[..p.len().min(80)]);
        }
    }

    format!("{}: {}", name, &tc.get("input").map(|v| v.to_string()).unwrap_or_default()[..80.min(tc.get("input").map(|v| v.to_string()).unwrap_or_default().len())])
}

/// 解析 Claude Code 会话 jsonl
pub fn parse_session(jsonl_path: &Path, max_turns: usize) -> Result<Vec<Message>, String> {
    let content = fs::read_to_string(jsonl_path)
        .map_err(|e| format!("读取会话文件失败: {}", e))?;

    let mut raw_messages: Vec<Message> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if entry_type != "user" && entry_type != "assistant" {
            continue;
        }

        // 跳过 sidechain
        if entry.get("isSidechain").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }

        let msg = entry.get("message").unwrap_or(&serde_json::Value::Null);
        let role = msg
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or(entry_type)
            .to_string();
        let content_raw = msg.get("content");

        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();

        match content_raw {
            Some(serde_json::Value::String(s)) => {
                if !s.trim().is_empty() {
                    text_parts.push(s.trim().to_string());
                }
            }
            Some(serde_json::Value::Array(arr)) => {
                for block in arr {
                    if let serde_json::Value::String(s) = block {
                        if !s.trim().is_empty() {
                            text_parts.push(s.trim().to_string());
                        }
                    } else if let serde_json::Value::Object(obj) = block {
                        let block_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        match block_type {
                            "text" => {
                                let t = obj
                                    .get("text")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .trim();
                                if !t.is_empty() {
                                    text_parts.push(t.to_string());
                                }
                            }
                            "tool_use" => {
                                tool_calls.push(block.clone());
                            }
                            "tool_result" => {
                                let content = obj.get("content");
                                let mut result_text = String::new();
                                match content {
                                    Some(serde_json::Value::String(s)) => {
                                        result_text = s.chars().take(200).collect();
                                    }
                                    Some(serde_json::Value::Array(arr2)) => {
                                        for sub in arr2 {
                                            if let serde_json::Value::Object(sub_obj) = sub {
                                                if sub_obj
                                                    .get("type")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    == "text"
                                                {
                                                    result_text.push_str(
                                                        sub_obj
                                                            .get("text")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or(""),
                                                    );
                                                    if result_text.len() > 200 {
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                if !result_text.trim().is_empty() {
                                    tool_calls.push(serde_json::json!({
                                        "name": "tool_result",
                                        "result_preview": result_text.trim().chars().take(150).collect::<String>()
                                    }));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        // 将工具调用摘要融入 content
        for tc in &tool_calls {
            let summary = tool_call_summary(tc);
            if tc.get("name").and_then(|v| v.as_str()) == Some("tool_result") {
                text_parts.push(format!("↩ {}", summary));
            } else {
                text_parts.push(format!("🔧 {}", summary));
            }
        }

        let text = text_parts.join("\n");
        if text.trim().is_empty() {
            continue;
        }

        // 对于 user：纯 tool_result 的消息精简
        let final_text = if role == "user"
            && !text.starts_with("↩")
            && text_parts.iter().all(|p| p.starts_with("↩") || p.starts_with("🔧"))
            && !text_parts.iter().any(|p| !p.starts_with("↩") && !p.starts_with("🔧"))
        {
            text
        } else {
            text
        };

        raw_messages.push(Message {
            role,
            content: final_text,
        });
    }

    // 合并连续同角色消息
    let mut merged: Vec<Message> = Vec::new();
    for msg in raw_messages {
        if let Some(last) = merged.last_mut() {
            if last.role == msg.role {
                last.content.push_str(&format!("\n{}", msg.content));
                continue;
            }
        }
        merged.push(msg);
    }

    Ok(merged.into_iter().take(max_turns).collect())
}

/// 从 Claude Code 提取完整上下文信息
pub fn extract_context(
    project_path: &str,
    session_id: Option<&str>,
    max_turns: usize,
) -> Result<ContextInfo, String> {
    let sessions = list_sessions(project_path)?;
    if sessions.is_empty() {
        return Err(format!(
            "未找到 Claude Code 会话记录，请确认项目路径 '{}' 下有 Claude Code 使用历史",
            project_path
        ));
    }

    let target = if let Some(sid) = session_id {
        sessions
            .iter()
            .find(|s| s.session_id == sid)
            .ok_or_else(|| format!("会话 ID '{}' 不存在", sid))?
            .session_id
            .clone()
    } else {
        sessions[0].session_id.clone()
    };

    let project_dir = get_project_dir(project_path);
    let jsonl_path = claude_home()
        .join("projects")
        .join(&project_dir)
        .join(format!("{}.jsonl", target));

    let messages = parse_session(&jsonl_path, max_turns)?;

    // 尝试提取 cwd 和 git 信息
    let content = fs::read_to_string(&jsonl_path).unwrap_or_default();
    let mut cwd = project_path.to_string();
    let mut git_branch: Option<String> = None;

    for line in content.lines() {
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            if entry.get("type").and_then(|v| v.as_str()) == Some("user") {
                if let Some(c) = entry.get("cwd").and_then(|v| v.as_str()) {
                    cwd = c.to_string();
                }
            }
            if let Some(b) = entry.get("gitBranch").and_then(|v| v.as_str()) {
                if b != "HEAD" {
                    git_branch = Some(b.to_string());
                }
            }
        }
    }

    Ok(ContextInfo {
        project_path: cwd.clone(),
        session_id: target,
        messages,
        cwd,
        git_branch,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub path: String,
    pub dir_name: String,
    pub has_sessions: bool,
}

/// 扫描 ~/.claude/projects/ 列出所有已知项目
pub fn list_projects() -> Result<Vec<ProjectEntry>, String> {
    let projects_dir = claude_home().join("projects");
    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut projects = Vec::new();
    let entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("读取项目目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let has_sessions = fs::read_dir(&path)
            .map(|mut d| {
                d.any(|e| {
                    e.as_ref()
                        .ok()
                        .and_then(|e| e.path().extension().and_then(|ext| ext.to_str().map(|s| s == "jsonl")))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        // 从 jsonl 文件中读取真实 cwd
        let cwd = {
            let mut found = None;
            if let Ok(mut entries) = fs::read_dir(&path) {
                while let Some(Ok(e)) = entries.next() {
                    if e.path().extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(e.path()) {
                        for line in content.lines().take(50) {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                                if let Some(c) = val.get("cwd").and_then(|v| v.as_str()) {
                                    found = Some(c.to_string());
                                    break;
                                }
                            }
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
            }
            found.unwrap_or(dir_name.clone())
        };

        projects.push(ProjectEntry {
            path: cwd,
            dir_name,
            has_sessions,
        });
    }

    projects.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(projects)
}
