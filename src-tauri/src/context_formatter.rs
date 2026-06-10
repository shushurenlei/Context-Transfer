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

    lines.push(format!("# 会话上下文迁移"));
    lines.push(String::new());
    lines.push(format!(
        "> 从会话 `{}...` 迁移",
        truncate(&context.session_id, 16)
    ));
    lines.push(format!("> 项目路径: `{}`", context.project_path));
    if let Some(ref branch) = context.git_branch {
        lines.push(format!("> Git 分支: `{}`", branch));
    }
    lines.push(format!(
        "> 迁移时间: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    ));
    lines.push(String::new());

    // 统计
    let user_questions: Vec<_> = context
        .messages
        .iter()
        .filter(|m| m.role == "user" && !m.content.starts_with("↩"))
        .collect();

    lines.push(format!("## 上下文摘要"));
    lines.push(String::new());
    lines.push(format!("- 用户主动提问 {} 次", user_questions.len()));
    lines.push(format!("- 对话轮次共 {} 条", context.messages.len()));
    if !user_questions.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "核心问题: {}",
            truncate(&user_questions[0].content, 200)
        ));
    }
    lines.push(String::new());

    // 对话历史
    lines.push(format!("## 对话历史"));
    lines.push(String::new());

    // 先构建所有消息块
    let mut blocks: Vec<Vec<String>> = Vec::new();
    let mut turn_num = 0usize;

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
        blocks.push(block);
    }

    // 从头部计算累计长度，若超限则跳过早期块，保留较近的
    let header_len: usize = lines.iter().map(|s| s.len() + 1).sum();
    let mut total = header_len;
    let limit = max_total_length.unwrap_or(usize::MAX);
    let mut skipped = 0usize;

    for (i, block) in blocks.iter().enumerate() {
        let block_len: usize = block.iter().map(|s| s.len() + 1).sum();
        if total + block_len > limit && i < blocks.len().saturating_sub(2) {
            skipped += 1;
        } else {
            for s in block {
                lines.push(s.clone());
            }
            total += block_len;
        }
    }

    if skipped > 0 {
        lines.push(format!(
            "> ⚠️ 上下文已达长度上限（{} 字符），较早期的 {} 条对话已省略",
            limit, skipped
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

    // 对话摘要 — 先收集再从尾部（最新）开始保留
    push_line!(format!("【对话摘要】"));
    let mut all_turns: Vec<String> = Vec::new();
    let mut turn = 0usize;

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
        all_turns.push(line);
    }

    let mut skipped = 0usize;
    for (i, line) in all_turns.iter().enumerate() {
        let line_len = line.len() + 1;
        if total + line_len > limit && i < all_turns.len().saturating_sub(2) {
            skipped += 1;
        } else {
            push_line!(line.clone());
        }
    }

    if skipped > 0 {
        push_line!(format!(
            "> ⚠️ 已达长度上限（{} 字符），早期 {} 条对话已省略",
            limit, skipped
        ));
    }

    push_line!(String::new());
    push_line!(format!("请基于以上上下文继续协助我。"));

    lines.join("\n")
}
