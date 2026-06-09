//! 将格式化后的上下文注入到 Codex CLI 的消费方式中

use crate::claud_extractor::ContextInfo;
use crate::context_formatter;
use std::fs;
use std::path::Path;
use std::process::Command;

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

/// 在新终端窗口中启动 Codex CLI
pub fn launch_codex(project_path: &str, model: Option<&str>) -> Result<u32, String> {
    let codex_cmd = match model {
        Some(m) => format!("codex --model '{}'", m.replace('\'', "'\\''")),
        None => "codex".to_string(),
    };

    let escaped_path = project_path.replace('\'', "'\\''");

    // 拼接 shell 命令
    let shell_cmd = format!("cd '{}' && source ~/.zshrc 2>/dev/null; {}", escaped_path, codex_cmd);

    // 判断 Terminal 是否已在运行，避免启动时多开一个默认窗口
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
        .map_err(|e| format!("启动终端 Codex 失败: {}", e))?;

    Ok(child.id())
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
    model: Option<&str>,
    max_content_length: usize,
) -> Result<MigrateResult, String> {
    match mode {
        "prompt" => {
            let prompt = context_formatter::format_as_prompt(context, max_content_length);
            copy_to_clipboard(&prompt)?;
            Ok(MigrateResult {
                success: true,
                message: "📋 上下文已复制到剪贴板，在 Codex 中粘贴即可".to_string(),
                filepath: None,
            })
        }
        "agents-md" => {
            let md = context_formatter::format_as_markdown(context, max_content_length);
            let filepath = inject_via_agents_md(&md, project_path, true)?;
            Ok(MigrateResult {
                success: true,
                message: format!("📝 上下文已写入 {}", filepath),
                filepath: Some(filepath),
            })
        }
        "auto" => {
            let md = context_formatter::format_as_markdown(context, max_content_length);
            let filepath = inject_via_agents_md(&md, project_path, true)?;
            let pid = launch_codex(project_path, model)?;
            Ok(MigrateResult {
                success: true,
                message: format!(
                    "📝 上下文已写入 {}，Codex 已启动 (PID: {})",
                    filepath, pid
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
) -> Result<String, String> {
    let prompt = context_formatter::format_as_prompt(context, max_content_length);
    copy_to_clipboard(&prompt)?;
    Ok(prompt)
}
