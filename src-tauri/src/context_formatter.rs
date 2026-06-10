//! 将提取的上下文格式化为目标工具可消费的形式

use crate::claud_extractor::ContextInfo;

/// 安全截断字符串（按字符边界）
fn truncate(s: &str, max_chars: usize) -> &str {
    if s.chars().count() <= max_chars {
        return s;
    }
    let idx = s
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    &s[..idx]
}

/// 将上下文格式化为 Markdown 文档
/// max_total_length: 总长度上限（字符数），None 表示不限制
pub fn format_as_markdown(
    context: &ContextInfo,
    max_content_length: usize,
    max_total_length: Option<usize>,
) -> String {
    let mut lines = Vec::new();
    let mut total = 0usize;

    macro_rules! push_line {
        ($s:expr) => {
            let s: String = $s;
            total += s.len() + 1;
            lines.push(s);
        };
    }

    push_line!(format!("# 会话上下文迁移"));
    push_line!(String::new());
    push_line!(format!(
        "> 从会话 `{}...` 迁移",
        truncate(&context.session_id, 16)
    ));
    push_line!(format!("> 项目路径: `{}`", context.project_path));
    if let Some(ref branch) = context.git_branch {
        push_line!(format!("> Git 分支: `{}`", branch));
    }
    push_line!(format!(
        "> 迁移时间: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    ));
    push_line!(String::new());

    // 统计
    let user_questions: Vec<_> = context
        .messages
        .iter()
        .filter(|m| m.role == "user" && !m.content.starts_with("↩"))
        .collect();

    push_line!(format!("## 上下文摘要"));
    push_line!(String::new());
    push_line!(format!("- 用户主动提问 {} 次", user_questions.len()));
    push_line!(format!("- 对话轮次共 {} 条", context.messages.len()));
    if !user_questions.is_empty() {
        push_line!(String::new());
        push_line!(format!(
            "核心问题: {}",
            truncate(&user_questions[0].content, 200)
        ));
    }
    push_line!(String::new());

    // 对话历史 — 如果超长则从最早的开始丢弃
    push_line!(format!("## 对话历史"));
    push_line!(String::new());

    let limit = max_total_length.unwrap_or(usize::MAX);
    let mut turn_num = 0usize;
    let mut trimmed = false;

    for msg in &context.messages {
        let content = if msg.content.chars().count() > max_content_length {
            format!("{}...[已截断]", truncate(&msg.content, max_content_length))
        } else {
            msg.content.clone()
        };

        let mut block = Vec::new();
        if msg.role == "user" {
            if msg.content.starts_with("↩") {
                block.push(format!("**工具输出**: {}", truncate(&content, 200)));
                block.push(String::new());
            } else {
                turn_num += 1;
                block.push(format!("### 用户 #{}", turn_num));
                block.push(String::new());
                block.push(content);
                block.push(String::new());
            }
        } else if msg.role == "assistant" {
            block.push(format!("**助手**:"));
            block.push(String::new());
            block.push(content);
            block.push(String::new());
        }

        let block_len: usize = block.iter().map(|s| s.len() + 1).sum();
        if total + block_len > limit && turn_num > 2 {
            trimmed = true;
            break;
        }
        for s in block {
            push_line!(s);
        }
    }

    if trimmed {
        push_line!(format!(
            "> ⚠️ 上下文已达长度上限（{} 字符），较早期的对话已省略。",
            limit
        ));
    }

    lines.join("\n")
}

/// 将上下文格式化为 prompt 文本
pub fn format_as_prompt(
    context: &ContextInfo,
    max_content_length: usize,
    max_total_length: Option<usize>,
) -> String {
    let limit = max_total_length.unwrap_or(usize::MAX);
    let mut lines = Vec::new();
    let mut total = 0usize;

    macro_rules! push_line {
        ($s:expr) => {
            let s: String = $s;
            total += s.len() + 1;
            lines.push(s);
        };
    }

    push_line!(format!(
        "[迁移的上下文] 请延续以下上下文继续协助我："
    ));
    push_line!(String::new());

    // 核心需求
    let first_user = context
        .messages
        .iter()
        .find(|m| m.role == "user" && !m.content.starts_with("↩"));

    if let Some(msg) = first_user {
        push_line!(format!(
            "【核心需求】{}",
            truncate(&msg.content, max_content_length)
        ));
        push_line!(String::new());
    }

    // 对话摘要（从新到旧，重要消息在前）
    push_line!(format!("【对话摘要】"));
    let mut turn = 0usize;
    let mut trimmed = false;

    for msg in &context.messages {
        let is_tool_result = msg.content.starts_with("↩");
        let snippet: String = msg.content.chars().take(200).collect();

        let line = if msg.role == "user" && !is_tool_result {
            turn += 1;
            format!("  用户#{}: {}", turn, snippet)
        } else if msg.role == "user" && is_tool_result {
            let short: String = msg.content.chars().take(80).collect();
            format!("  ↩ {}", short)
        } else if msg.role == "assistant" {
            format!("  助手: {}", snippet)
        } else {
            continue;
        };

        if total + line.len() + 1 > limit && turn > 2 {
            trimmed = true;
            break;
        }
        push_line!(line);
    }

    if trimmed {
        push_line!(format!(
            "> ⚠️ 已达长度上限（{} 字符），早期对话已省略。",
            limit
        ));
    }

    push_line!(String::new());
    push_line!(format!("请基于以上上下文继续协助我。"));

    lines.join("\n")
}
