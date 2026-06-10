//! Context Transfer - Claude Code ↔ Codex 上下文迁移工具

mod claud_extractor;
mod codex_extractor;
mod commands;
mod codex_injector;
mod context_formatter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_project,
            commands::list_projects,
            commands::list_sessions,
            commands::extract_context,
            commands::migrate,
            commands::copy_prompt,
            commands::cleanup,
            commands::export_context,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Context Transfer 失败");
}
