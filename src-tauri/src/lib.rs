// 新架构模块
mod core;
mod features;
mod infrastructure;

// 重新导出核心类型
pub use core::AppState;

use core::Feature;
use features::{settings::SettingsFeature, tab::TabFeature, window::WindowFeature};
use infrastructure::database::{init_db, DatabaseRegistry};
use std::sync::Arc;
use tauri::Manager;

/// 初始化所有 Features
fn init_features() -> Vec<Arc<dyn Feature>> {
    vec![
        SettingsFeature::new(),
        Arc::new(TabFeature::new()),
        Arc::new(WindowFeature::new()),
    ]
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle();

            // 初始化数据库
            let db = tauri::async_runtime::block_on(init_db(&handle))
                .map_err(|e| format!("Failed to init database: {}", e))?;

            // 初始化所有 Features
            let features = init_features();

            // 创建数据库注册表并执行所有 Migrations
            let mut db_registry = DatabaseRegistry::new();
            for feature in &features {
                feature.register_database(&mut db_registry);
            }

            tauri::async_runtime::block_on(db_registry.run_migrations(&db))
                .map_err(|e| format!("Failed to run migrations: {}", e))?;

            // 创建 AppState
            let mut state = AppState::new(handle.clone(), db, features.clone());

            // 注册 WebSocket Handlers
            {
                let mut registry =
                    tauri::async_runtime::block_on(state.webserver_manager().registry_mut());
                for feature in &features {
                    feature.register_ws_handlers(&mut registry);
                }
            }

            // 构建并设置 Tray Registry
            let tray_registry = core::registry::tray::build_tray_registry();
            state.set_tray_registry(tray_registry);

            // 初始化所有 Features
            for feature in &features {
                tauri::async_runtime::block_on(feature.initialize(&state)).map_err(|e| {
                    format!("Failed to initialize feature '{}': {}", feature.name(), e)
                })?;
            }

            // 托管状态
            app.manage(state);

            // 后初始化（执行需要访问已托管状态的逻辑）
            if let Some(app_state) = app.try_state::<AppState>() {
                tauri::async_runtime::block_on(app_state.post_initialize(&handle))
                    .map_err(|e| format!("Post-initialization failed: {}", e))?;
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // 仅对主窗口进行拦截，其它窗口正常关闭
                if window.label() == "main" {
                    features::window::manager::handle_window_close_request(window, api);
                }
            }
            // 监听移动和调整大小事件，同步内容窗口位置
            tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                if window.label() == "main" {
                    features::window::manager::on_main_window_moved(window.app_handle());
                }
            }
            // 监听焦点事件，处理自动显示/隐藏
            tauri::WindowEvent::Focused(focused) => {
                features::window::manager::handle_focus_change(window.app_handle(), window.label(), *focused);
            }
            _ => {}
        })
        .invoke_handler(core::registry::commands::get_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
