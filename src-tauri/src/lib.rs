mod commands;
mod db;
mod services;

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

use services::ai::HttpTransport;
use services::secrets::Secrets;

/// 全局状态：一条 SQLite 连接 + 数据库文件路径。
/// 桌面端是单进程单窗口场景，用 Mutex 串行化访问即可。
pub struct AppState {
    pub db: Mutex<Connection>,
    pub path: PathBuf,
    /// API Key 的加解密
    pub secrets: Secrets,
    /// AI 请求的传输层
    pub transport: HttpTransport,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 首次启动：建目录 -> 建库 -> 跑迁移建表
            let path = db::resolve_db_path(app.handle())?;
            let conn = db::connect(&path)?;

            let data_dir = path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            let secrets = Secrets::load_or_create(&data_dir);
            let transport = HttpTransport::new(60)?;

            app.manage(AppState {
                db: Mutex::new(conn),
                path,
                secrets,
                transport,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::db_status,
            commands::db_describe,
            commands::db_list,
            commands::db_get,
            commands::db_insert,
            commands::db_update,
            commands::db_delete,
            commands::db_count,
            commands::db_self_test,
            commands::setting_get,
            commands::setting_set,
            commands::setting_all,
            commands::ai_usage_record,
            commands::ai_usage_summary,
            services::ai::commands::ai_providers,
            services::ai::commands::ai_settings,
            services::ai::commands::ai_quota,
            services::ai::commands::ai_save_key,
            services::ai::commands::ai_delete_key,
            services::ai::commands::ai_select,
            services::ai::commands::save_nickname,
            services::ai::commands::ai_test_connection,
            services::ai::commands::ai_chat,
            services::ai::commands::ai_usage_history,
            services::personas::commands::persona_categories,
            services::personas::commands::persona_prompt_block,
            services::personas::commands::ai_extract_persona,
            services::materials::commands::material_import,
            services::materials::commands::material_pick,
            services::materials::commands::material_delete,
            services::materials::commands::material_categories,
            services::materials::commands::material_category_create,
            services::materials::commands::material_category_delete,
            services::materials::commands::material_read_text,
            services::materials::commands::material_save_thumbnail,
            services::materials::commands::material_prompt_block,
            services::materials::commands::material_library_info,
            services::materials::commands::ai_describe_material,
            services::writing::commands::writing_genres,
            services::writing::commands::ai_write_draft,
            services::writing::commands::ai_revise_draft,
            services::writing::commands::writing_export,
            services::memory::commands::memory_list,
            services::memory::commands::memory_search,
            services::memory::commands::memory_stats,
            services::memory::commands::memory_delete,
            services::memory::commands::memory_reindex,
            services::memory::commands::memory_export,
            services::memory::commands::memory_import,
            services::memory::commands::memory_prompt_block,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
