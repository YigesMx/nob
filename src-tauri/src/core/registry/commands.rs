use tauri::ipc::Invoke;

/// 获取所有命令的处理器
///
/// 封装 Tauri 的 generate_handler! 宏，统一管理所有命令注册
pub fn get_handler() -> impl Fn(Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // Settings Feature Commands
        crate::features::settings::api::commands::get_theme_preference,
        crate::features::settings::api::commands::set_theme_preference,
        // Tab Feature Commands
        crate::features::tab::api::commands::tabs_list,
        crate::features::tab::api::commands::tabs_create,
        crate::features::tab::api::commands::tabs_update,
        crate::features::tab::api::commands::tabs_activate,
        crate::features::tab::api::commands::tabs_close,
        crate::features::tab::api::commands::tabs_reorder,
        crate::features::tab::api::commands::tabs_activate_next,
        crate::features::tab::api::commands::tabs_activate_previous,
        crate::features::tab::api::commands::tabs_close_active,
        // WebServer Commands
        crate::infrastructure::webserver::api::commands::start_web_server,
        crate::infrastructure::webserver::api::commands::stop_web_server,
        crate::infrastructure::webserver::api::commands::web_server_status,
    ]
}
