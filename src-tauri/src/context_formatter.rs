//! 将提取的上下文格式化为 Codex CLI 可消费的形式

use crate::claud_extractor::ContextInfo;

/// 将上下文格式化为 Markdown 文档
pub fn format_as_markdown(context: &ContextInfo, max_content_length: usize) -> String {
    let mut lines = Vec::new();

    lines.push("# Claude Code 会话上下文迁移".to_string());
    lines.push(String::new());
    lines.push(format!(
        "> 从 Claude Code 会话 `{}...` 迁移",
        &context.session_id[..16.min(context.session_id.len())]
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

    lines.push("## 上下文摘要".to_string());
    lines.push(String::new());
    lines.push(format!("- 用户主动提问 {} 次", user_questions.len()));
    lines.push(format!("- 对话轮次共 {} 条", context.messages.len()));
    if !user_questions.is_empty() {
        lines.push(String::new());
        let first = &user_questions[0].content;
        lines.push(format!(
            "核心问题: {}",
            &first[..200.min(first.len())]
        ));
    }
    lines.push(String::new());

    // 对话历史
    lines.push("## 对话历史".to_string());
    lines.push(String::new());

    let mut turn_num = 0usize;

    for msg in &context.messages {
        let content = if msg.content.len() > max_content_length {
            format!("{}...[已截断]", &msg.content[..max_content_length])
        } else {
            msg.content.clone()
        };

        if msg.role == "user" {
            if msg.content.starts_with("↩") {
                lines.push(format!("**工具输出**: {}", &content[..200.min(content.len())]));
                lines.push(String::new());
            } else {
                turn_num += 1;
                lines.push(format!("### 用户 #{}", turn_num));
                lines.push(String::new());
                lines.push(content);
                lines.push(String::new());
            }
        } else if msg.role == "assistant" {
            lines.push("**助手**:".to_string());
            lines.push(String::new());
            lines.push(content);
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

/// 将上下文格式化为 Codex 首个 prompt 文本
pub fn format_as_prompt(context: &ContextInfo, max_content_length: usize) -> String {
    let mut lines = Vec::new();

    lines.push(
        "[从 Claude Code 迁移的上下文] 请延续以下上下文继续协助我：".to_string(),
    );
    lines.push(String::new());

    // 核心需求
    let first_user = context
        .messages
        .iter()
        .find(|m| m.role == "user" && !m.content.starts_with("↩"));

    if let Some(msg) = first_user {
        let text = &msg.content;
        lines.push(format!(
            "【核心需求】{}",
            &text[..max_content_length.min(text.len())]
        ));
        lines.push(String::new());
    }

    // 精简对话摘要
    lines.push("【对话摘要】".to_string());
    let mut turn = 0usize;

    for msg in &context.messages {
        let is_tool_result = msg.content.starts_with("↩");
        let snippet: String = msg.content.chars().take(200).collect();

        if msg.role == "user" && !is_tool_result {
            turn += 1;
            lines.push(format!("  用户#{}: {}", turn, snippet));
        } else if msg.role == "user" && is_tool_result {
            let short: String = msg.content.chars().take(80).collect();
            lines.push(format!("  ↩ {}", short));
        } else if msg.role == "assistant" {
            lines.push(format!("  助手: {}", snippet));
        }
    }

    lines.push(String::new());
    lines.push("请基于以上上下文继续协助我。".to_string());

    lines.join("\n")
}
