//! 将格式化后的上下文注入到 Codex CLI / Claude Code 的消费方式中

use crate::claud_extractor::ContextInfo;
use crate::context_formatter;
use std::fs;
use std::path::Path;
use std::process::Command;

const MIGRATION_START_MARKER: &str = "<!-- context-transfer:begin -->";
const MIGRATION_END_MARKER: &str = "<!-- context-transfer:end -->";
const MIGRATION_HEADING: &str = "# 会话上下文迁移";
const LEGACY_MIGRATION_HEADINGS: [&str; 3] = [
    MIGRATION_HEADING,
    "# Claude Code 会话上下文迁移",
    "# Codex 会话上下文迁移",
];

fn wrap_migration_block(context_md: &str) -> String {
    format!(
        "{}\n{}\n{}",
        MIGRATION_START_MARKER,
        context_md.trim_end(),
        MIGRATION_END_MARKER
    )
}

fn separator_start_before(content: &str, marker_idx: usize) -> usize {
    let prefix = &content[..marker_idx];
    let trimmed = prefix.trim_end();

    if trimmed.ends_with("---") {
        trimmed.rfind("---").unwrap_or(marker_idx)
    } else {
        marker_idx
    }
}

fn join_after_removal(prefix: &str, suffix: &str) -> String {
    let prefix = prefix.trim_end();
    let suffix = suffix.trim_start();

    match (prefix.is_empty(), suffix.is_empty()) {
        (true, true) => String::new(),
        (true, false) => suffix.to_string(),
        (false, true) => prefix.to_string(),
        (false, false) => format!("{}\n\n{}", prefix, suffix),
    }
}

fn remove_marked_migration_blocks(content: &str) -> (String, bool) {
    let mut cleaned = content.to_string();
    let mut removed = false;

    while let Some(start) = cleaned.find(MIGRATION_START_MARKER) {
        let search_from = start + MIGRATION_START_MARKER.len();
        let end = cleaned[search_from..]
            .find(MIGRATION_END_MARKER)
            .map(|idx| search_from + idx + MIGRATION_END_MARKER.len())
            .unwrap_or(cleaned.len());
        let removal_start = separator_start_before(&cleaned, start);
        cleaned = join_after_removal(&cleaned[..removal_start], &cleaned[end..]);
        removed = true;
    }

    (cleaned, removed)
}

fn remove_legacy_migration_block(content: &str) -> (String, bool) {
    let legacy_idx = LEGACY_MIGRATION_HEADINGS
        .iter()
        .filter_map(|heading| content.find(heading))
        .min();

    match legacy_idx {
        Some(idx) => {
            let removal_start = separator_start_before(content, idx);
            (content[..removal_start].trim_end().to_string(), true)
        }
        None => (content.to_string(), false),
    }
}

fn remove_existing_migration_blocks(content: &str) -> (String, bool) {
    let (without_marked, removed_marked) = remove_marked_migration_blocks(content);
    let (without_legacy, removed_legacy) = remove_legacy_migration_block(&without_marked);

    (without_legacy, removed_marked || removed_legacy)
}

/// 构建跨平台 shell 命令
fn build_shell_cmd(project_path: &str, cli_cmd: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!(
            "cd '{}' && source ~/.zshrc 2>/dev/null; {}",
            project_path.replace('\'', "'\\''"),
            cli_cmd,
        )
    }
    #[cfg(target_os = "windows")]
    {
        format!(
            "cd /d \"{}\" && {}",
            project_path.replace('\"', "\\\""),
            cli_cmd,
        )
    }
    #[cfg(target_os = "linux")]
    {
        format!(
            "cd '{}' && [ -f ~/.bashrc ] && source ~/.bashrc 2>/dev/null; {}",
            project_path.replace('\'', "'\\''"),
            cli_cmd,
        )
    }
}

/// 在新终端窗口中执行命令（跨平台）
fn launch_in_terminal(shell_cmd: &str) -> Result<u32, String> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "tell app \"Terminal\"\n\
             \x20 if (count of windows) = 0 then\n\
             \x20 \x20 do script \"{0}\"\n\
             \x20 else\n\
             \x20 \x20 do script \"{0}\" in front window\n\
             \x20 end if\n\
             \x20 activate\n\
             end tell",
            shell_cmd.replace('\"', "\\\"")
        );
        let child = Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;
        Ok(child.id())
    }
    #[cfg(target_os = "windows")]
    {
        let child = Command::new("cmd")
            .args(["/c", "start", "Context Transfer", "cmd", "/k", shell_cmd])
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;
        Ok(child.id())
    }
    #[cfg(target_os = "linux")]
    {
        let terminals = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
        ];
        for term in &terminals {
            let args: &[&str] = if *term == "gnome-terminal" {
                &["--", "bash", "-c", shell_cmd]
            } else {
                &["-e", shell_cmd]
            };
            if let Ok(child) = Command::new(term).args(args).spawn() {
                return Ok(child.id());
            }
        }
        Err("未找到可用的终端模拟器".to_string())
    }
}

/// 通过 arboard 复制文本到系统剪贴板（跨平台）
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("打开剪贴板失败: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("写入剪贴板失败: {}", e))?;
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
            existing = remove_existing_migration_blocks(&existing).0;
        }
    }

    let migration_block = wrap_migration_block(context_md);
    let combined = if !existing.trim().is_empty() {
        format!("{}\n\n---\n\n{}", existing.trim_end(), migration_block)
    } else {
        migration_block
    };

    fs::write(&agents_md_path, combined).map_err(|e| format!("写入 AGENTS.md 失败: {}", e))?;

    Ok(agents_md_path.to_string_lossy().to_string())
}

/// 清理 AGENTS.md 中的迁移上下文段
pub fn cleanup_agents_md(project_path: &str) -> Result<bool, String> {
    let agents_md_path = Path::new(project_path).join("AGENTS.md");
    if !agents_md_path.exists() {
        return Ok(false);
    }

    let content =
        fs::read_to_string(&agents_md_path).map_err(|e| format!("读取 AGENTS.md 失败: {}", e))?;

    let (cleaned, removed) = remove_existing_migration_blocks(&content);
    if !removed {
        return Ok(false);
    }

    fs::write(&agents_md_path, cleaned).map_err(|e| format!("写入 AGENTS.md 失败: {}", e))?;
    Ok(true)
}

/// 通过写入项目 CLAUDE.md 注入上下文（给 Claude Code 使用）
pub fn inject_via_claude_md(
    context_md: &str,
    project_path: &str,
    cleanup: bool,
) -> Result<String, String> {
    let md_path = Path::new(project_path).join("CLAUDE.md");

    let mut existing = String::new();
    if md_path.exists() {
        existing =
            fs::read_to_string(&md_path).map_err(|e| format!("读取 CLAUDE.md 失败: {}", e))?;

        if cleanup {
            existing = remove_existing_migration_blocks(&existing).0;
        }
    }

    let migration_block = wrap_migration_block(context_md);
    let combined = if !existing.trim().is_empty() {
        format!("{}\n\n---\n\n{}", existing.trim_end(), migration_block)
    } else {
        migration_block
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

    let (cleaned, removed) = remove_existing_migration_blocks(&content);
    if !removed {
        return Ok(false);
    }

    fs::write(&md_path, cleaned).map_err(|e| format!("写入 CLAUDE.md 失败: {}", e))?;
    Ok(true)
}

/// 在新终端窗口中启动 Codex CLI
pub fn launch_codex(project_path: &str, model: Option<&str>) -> Result<u32, String> {
    let codex_cmd = match model {
        Some(m) => format!("codex --model '{}'", m.replace('\'', "'\\''")),
        None => "codex".to_string(),
    };

    let shell_cmd = build_shell_cmd(project_path, &codex_cmd);
    launch_in_terminal(&shell_cmd)
}

/// 在新终端窗口中启动 Claude Code
pub fn launch_claude(project_path: &str) -> Result<u32, String> {
    let shell_cmd = build_shell_cmd(project_path, "claude");
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
    max_total_length: Option<usize>,
) -> Result<MigrateResult, String> {
    let is_codex_to_claude = direction == "codex-to-claude";

    match mode {
        "prompt" => {
            let prompt =
                context_formatter::format_as_prompt(context, max_content_length, max_total_length);
            copy_to_clipboard(&prompt)?;
            let target = if is_codex_to_claude {
                "Claude Code"
            } else {
                "Codex"
            };
            Ok(MigrateResult {
                success: true,
                message: format!("📋 上下文已复制到剪贴板，在 {} 中粘贴即可", target),
                filepath: None,
            })
        }
        "agents-md" => {
            let md = context_formatter::format_as_markdown(
                context,
                max_content_length,
                max_total_length,
            );
            let target_path = if is_codex_to_claude {
                &context.cwd
            } else {
                project_path
            };
            let filepath = if is_codex_to_claude {
                inject_via_claude_md(&md, target_path, true)?
            } else {
                inject_via_agents_md(&md, target_path, true)?
            };
            Ok(MigrateResult {
                success: true,
                message: format!("📝 上下文已写入 {}", filepath),
                filepath: Some(filepath),
            })
        }
        "auto" => {
            let md = context_formatter::format_as_markdown(
                context,
                max_content_length,
                max_total_length,
            );
            let target_path = if is_codex_to_claude {
                &context.cwd
            } else {
                project_path
            };
            let filepath = if is_codex_to_claude {
                inject_via_claude_md(&md, target_path, true)?
            } else {
                inject_via_agents_md(&md, target_path, true)?
            };
            let pid = if is_codex_to_claude {
                launch_claude(target_path)?
            } else {
                launch_codex(target_path, model)?
            };
            let app = if is_codex_to_claude {
                "Claude Code"
            } else {
                "Codex"
            };
            Ok(MigrateResult {
                success: true,
                message: format!(
                    "📝 上下文已写入 {}，{} 已启动 (PID: {})",
                    filepath, app, pid
                ),
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
    max_total_length: Option<usize>,
) -> Result<String, String> {
    let prompt = context_formatter::format_as_prompt(context, max_content_length, max_total_length);
    copy_to_clipboard(&prompt)?;
    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("context_transfer_test_{}", nanos));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn cleanup_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn inject_agents_md_replaces_legacy_block_instead_of_appending() {
        let dir = temp_project_dir();
        let agents_path = dir.join("AGENTS.md");
        fs::write(
            &agents_path,
            "# 项目规则\n\n保留内容\n\n---\n\n# 会话上下文迁移\nold block",
        )
        .unwrap();

        inject_via_agents_md("# 会话上下文迁移\nnew block", dir.to_str().unwrap(), true).unwrap();

        let content = fs::read_to_string(&agents_path).unwrap();
        assert!(content.contains("# 项目规则"));
        assert!(content.contains("new block"));
        assert!(!content.contains("old block"));
        assert!(content.contains(MIGRATION_START_MARKER));
        assert!(content.contains(MIGRATION_END_MARKER));
        assert_eq!(content.matches(MIGRATION_HEADING).count(), 1);

        cleanup_dir(&dir);
    }

    #[test]
    fn cleanup_agents_md_removes_marked_block() {
        let dir = temp_project_dir();
        let agents_path = dir.join("AGENTS.md");
        fs::write(
            &agents_path,
            format!(
                "# 项目规则\n\n保留内容\n\n---\n\n{}\n# 会话上下文迁移\nold block\n{}",
                MIGRATION_START_MARKER, MIGRATION_END_MARKER
            ),
        )
        .unwrap();

        let cleaned = cleanup_agents_md(dir.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&agents_path).unwrap();
        assert!(cleaned);
        assert_eq!(content.trim_end(), "# 项目规则\n\n保留内容");

        cleanup_dir(&dir);
    }

    #[test]
    fn cleanup_claude_md_removes_legacy_codex_heading_block() {
        let dir = temp_project_dir();
        let claude_path = dir.join("CLAUDE.md");
        fs::write(
            &claude_path,
            "# Claude 规则\n\n---\n\n# Codex 会话上下文迁移\nold block",
        )
        .unwrap();

        let cleaned = cleanup_claude_md(dir.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&claude_path).unwrap();
        assert!(cleaned);
        assert_eq!(content.trim_end(), "# Claude 规则");

        cleanup_dir(&dir);
    }
}
