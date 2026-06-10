//! 将格式化后的上下文注入到 Codex CLI / Claude Code 的消费方式中

use crate::claud_extractor::ContextInfo;
use crate::context_formatter;
use std::fs;
use std::path::Path;
use std::process::Command;

/// 通用终端启动脚本生成
fn terminal_script(shell_cmd: &str) -> String {
    format!(
        "tell app \"Terminal\"\n\
         \x20 if (count of windows) = 0 then\n\
         \x20 \x20 do script \"{0}\"\n\
         \x20 else\n\
         \x20 \x20 do script \"{0}\" in front window\n\
         \x20 end if\n\
         \x20 activate\n\
         end tell",
        shell_cmd.replace('\"', "\\\"")
    )
}

/// 通过 osascript 在新终端窗口中执行命令
fn launch_in_terminal(shell_cmd: &str) -> Result<u32, String> {
    let script = terminal_script(shell_cmd);
    let child = Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("启动终端失败: {}", e))?;
    Ok(child.id())
}

/// 通过 pbcopy 复制文本到系统剪贴板 (macOS)
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 pbcopy 失败: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("写入剪贴板失败: {}", e))?;
    }

    child
        .wait()
        .map_err(|e| format!("等待 pbcopy 失败: {}", e))?;
    Ok(())
}

/// 通过写入项目 AGENTS.md 注入上下文
pub fn inject_via_agents_md(
    context_md: &str,
    project_path: &str,
    cleanup: bool,
) -> Result<String, String> {
    let agents_md_path = Path::new(project_path).join("AGENTS.md");

    let mut existing = String::new();
    if agents_md_path.exists() {
        existing = fs::read_to_string(&agents_md_path)
            .map_err(|e| format!("读取 AGENTS.md 失败: {}", e))?;

        if cleanup {
            // 清理之前的迁移段
            if let Some(idx) = existing.find("# Claude Code 会话上下文迁移") {
                if let Some(sep_idx) = existing[..idx].rfind("---") {
                    existing = existing[..sep_idx].trim_end().to_string();
                } else {
                    existing = existing[..idx].trim_end().to_string();
                }
            }
        }
    }

    let combined = if !existing.trim().is_empty() {
        format!("{}\n\n---\n\n{}", existing.trim_end(), context_md)
    } else {
        context_md.to_string()
    };

    fs::write(&agents_md_path, combined)
        .map_err(|e| format!("写入 AGENTS.md 失败: {}", e))?;

    Ok(agents_md_path.to_string_lossy().to_string())
}

/// 清理 AGENTS.md 中的迁移上下文段
pub fn cleanup_agents_md(project_path: &str) -> Result<bool, String> {
    let agents_md_path = Path::new(project_path).join("AGENTS.md");
    if !agents_md_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&agents_md_path)
        .map_err(|e| format!("读取 AGENTS.md 失败: {}", e))?;

    if !content.contains("# Claude Code 会话上下文迁移") {
        return Ok(false);
    }

    if let Some(idx) = content.find("# Claude Code 会话上下文迁移") {
        let cleaned = if let Some(sep_idx) = content[..idx].rfind("---") {
            format!("{}\n", content[..sep_idx].trim_end())
        } else {
            format!("{}\n", content[..idx].trim_end())
        };

        fs::write(&agents_md_path, cleaned)
            .map_err(|e| format!("写入 AGENTS.md 失败: {}", e))?;
        return Ok(true);
    }

    Ok(false)
}

/// 通过写入项目 CLAUDE.md 注入上下文（给 Claude Code 使用）
pub fn inject_via_claude_md(
    context_md: &str,
    project_path: &str,
    cleanup: bool,
) -> Result<String, String> {
    let md_path = Path::new(project_path).join("CLAUDE.md");
    let marker = "# Codex 会话上下文迁移";

    let mut existing = String::new();
    if md_path.exists() {
        existing =
            fs::read_to_string(&md_path).map_err(|e| format!("读取 CLAUDE.md 失败: {}", e))?;

        if cleanup {
            if let Some(idx) = existing.find(marker) {
                if let Some(sep_idx) = existing[..idx].rfind("---") {
                    existing = existing[..sep_idx].trim_end().to_string();
                } else {
                    existing = existing[..idx].trim_end().to_string();
                }
            }
        }
    }

    let combined = if !existing.trim().is_empty() {
        format!("{}\n\n---\n\n{}", existing.trim_end(), context_md)
    } else {
        context_md.to_string()
    };

    fs::write(&md_path, combined).map_err(|e| format!("写入 CLAUDE.md 失败: {}", e))?;

    Ok(md_path.to_string_lossy().to_string())
}

/// 清理 CLAUDE.md 中的迁移上下文段
pub fn cleanup_claude_md(project_path: &str) -> Result<bool, String> {
    let md_path = Path::new(project_path).join("CLAUDE.md");
    if !md_path.exists() {
        return Ok(false);
    }

    let content =
        fs::read_to_string(&md_path).map_err(|e| format!("读取 CLAUDE.md 失败: {}", e))?;
    let marker = "# Codex 会话上下文迁移";

    if !content.contains(marker) {
        return Ok(false);
    }

    if let Some(idx) = content.find(marker) {
        let cleaned = if let Some(sep_idx) = content[..idx].rfind("---") {
            format!("{}\n", content[..sep_idx].trim_end())
        } else {
            format!("{}\n", content[..idx].trim_end())
        };

        fs::write(&md_path, cleaned).map_err(|e| format!("写入 CLAUDE.md 失败: {}", e))?;
        return Ok(true);
    }

    Ok(false)
}

/// 在新终端窗口中启动 Codex CLI
pub fn launch_codex(project_path: &str, model: Option<&str>) -> Result<u32, String> {
    let codex_cmd = match model {
        Some(m) => format!("codex --model '{}'", m.replace('\'', "'\\''")),
        None => "codex".to_string(),
    };

    let shell_cmd = format!(
        "cd '{}' && source ~/.zshrc 2>/dev/null; {}",
        project_path.replace('\'', "'\\''"),
        codex_cmd,
    );

    launch_in_terminal(&shell_cmd)
}

/// 在新终端窗口中启动 Claude Code
pub fn launch_claude(project_path: &str) -> Result<u32, String> {
    let shell_cmd = format!(
        "cd '{}' && source ~/.zshrc 2>/dev/null; claude",
        project_path.replace('\'', "'\\''"),
    );

    launch_in_terminal(&shell_cmd)
}

/// 执行迁移操作
pub struct MigrateResult {
    pub success: bool,
    pub message: String,
    pub filepath: Option<String>,
}

pub fn do_migrate(
    context: &ContextInfo,
    project_path: &str,
    mode: &str,
    direction: &str,
    model: Option<&str>,
    max_content_length: usize,
) -> Result<MigrateResult, String> {
    let is_codex_to_claude = direction == "codex-to-claude";

    match mode {
        "prompt" => {
            let prompt = context_formatter::format_as_prompt(context, max_content_length);
            copy_to_clipboard(&prompt)?;
            let target = if is_codex_to_claude { "Claude Code" } else { "Codex" };
            Ok(MigrateResult {
                success: true,
                message: format!("📋 上下文已复制到剪贴板，在 {} 中粘贴即可", target),
                filepath: None,
            })
        }
        "agents-md" => {
            let md = context_formatter::format_as_markdown(context, max_content_length);
            let filepath = if is_codex_to_claude {
                inject_via_claude_md(&md, project_path, true)?
            } else {
                inject_via_agents_md(&md, project_path, true)?
            };
            Ok(MigrateResult {
                success: true,
                message: format!("📝 上下文已写入 {}", filepath),
                filepath: Some(filepath),
            })
        }
        "auto" => {
            let md = context_formatter::format_as_markdown(context, max_content_length);
            let filepath = if is_codex_to_claude {
                inject_via_claude_md(&md, project_path, true)?
            } else {
                inject_via_agents_md(&md, project_path, true)?
            };
            let pid = if is_codex_to_claude {
                launch_claude(project_path)?
            } else {
                launch_codex(project_path, model)?
            };
            let app = if is_codex_to_claude { "Claude Code" } else { "Codex" };
            Ok(MigrateResult {
                success: true,
                message: format!("📝 上下文已写入 {}，{} 已启动 (PID: {})", filepath, app, pid),
                filepath: Some(filepath),
            })
        }
        _ => Err(format!("未知迁移模式: {}", mode)),
    }
}

/// 生成 prompt 并复制到剪贴板
pub fn copy_prompt(
    context: &ContextInfo,
    max_content_length: usize,
) -> Result<String, String> {
    let prompt = context_formatter::format_as_prompt(context, max_content_length);
    copy_to_clipboard(&prompt)?;
    Ok(prompt)
}
