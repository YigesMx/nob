use tauri::State;

use crate::core::AppState;
use crate::features::tab::core::models::{
    CreateTabPayload, ReorderTabsPayload, Tab, UpdateTabPayload,
};
use crate::features::tab::core::service::TabService;
use crate::features::window::manager as window_manager;
use serde_json::json;
use tauri::Emitter;

#[tauri::command]
pub async fn tabs_list(app_state: State<'_, AppState>) -> Result<Vec<Tab>, String> {
    TabService::list(app_state.db())
        .await
        .map(|tabs| tabs.into_iter().map(Tab::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tabs_create(
    app_state: State<'_, AppState>,
    payload: CreateTabPayload,
) -> Result<Tab, String> {
    let tab = TabService::create(app_state.db(), payload)
        .await
        .map(Tab::from)
        .map_err(|e| e.to_string())?;

    // 打开或导航内容窗口
    let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url), false);
    let _ = emit_tab_event(&app_state, "created", json!({ "tab": tab.clone() }));

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_update(
    app_state: State<'_, AppState>,
    payload: UpdateTabPayload,
) -> Result<Option<Tab>, String> {
    let updated = TabService::update(app_state.db(), payload)
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = updated {
        let _ = emit_tab_event(&app_state, "updated", json!({ "tab": tab }));
    }

    Ok(updated)
}

#[tauri::command]
pub async fn tabs_activate(app_state: State<'_, AppState>, id: String) -> Result<Option<Tab>, String> {
    // 获取当前缓存的 URL，用于判断是否需要导航
    let current_cached_url = window_manager::get_current_url();

    let tab = TabService::activate(app_state.db(), &id)
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = tab {
        // 如果 URL 相同，传入 None 以避免刷新；否则传入 Some(url) 进行导航
        let should_navigate = match current_cached_url {
            Some(ref cached) => cached != &tab.url,
            None => true,
        };
        let url_arg = if should_navigate { Some(tab.url.as_str()) } else { None };

        let _ = window_manager::present_content_window(&app_state.app_handle(), url_arg, false);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    } else {
        // 如果 activate 返回 None，可能是因为已经是 active 状态
        // 这种情况下，我们仍然需要确保窗口显示（解决再次点击无法打开的问题）
        if let Ok(Some(current_tab)) = TabService::get(app_state.db(), &id).await {
             let tab = Tab::from(current_tab);
             
             let should_navigate = match current_cached_url {
                Some(ref cached) => cached != &tab.url,
                None => true,
            };
            let url_arg = if should_navigate { Some(tab.url.as_str()) } else { None };

             let _ = window_manager::present_content_window(&app_state.app_handle(), url_arg, false);
        }
    }

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_close(app_state: State<'_, AppState>, id: String) -> Result<Option<Tab>, String> {
    let activated = TabService::close(app_state.db(), &id)
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    let _ = emit_tab_event(&app_state, "closed", json!({ "id": id }));

    if let Some(ref tab) = activated {
        let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url), false);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    } else {
        // 如果没有激活的标签页，隐藏内容窗口
        window_manager::hide_content_window(&app_state.app_handle());
        window_manager::clear_current_url();
    }

    Ok(activated)
}

#[tauri::command]
pub async fn tabs_reorder(
    app_state: State<'_, AppState>,
    payload: ReorderTabsPayload,
) -> Result<(), String> {
    TabService::reorder(app_state.db(), payload)
        .await
        .map_err(|e| e.to_string())
        .map(|_| {
            let _ = emit_tab_event(
                &app_state,
                "reordered",
                json!({ "at": chrono::Utc::now().to_rfc3339() }),
            );
        })
}

#[tauri::command]
pub async fn tabs_activate_next(app_state: State<'_, AppState>) -> Result<Option<Tab>, String> {
    let tab = TabService::activate_next(app_state.db())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = tab {
        let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url), false);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    }

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_activate_previous(app_state: State<'_, AppState>) -> Result<Option<Tab>, String> {
    let tab = TabService::activate_previous(app_state.db())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(ref tab) = tab {
        let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url), false);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    }

    Ok(tab)
}

#[tauri::command]
pub async fn tabs_close_active(app_state: State<'_, AppState>) -> Result<Option<Tab>, String> {
    let activated = TabService::close_active(app_state.db())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    let _ = emit_tab_event(&app_state, "closed", json!({ "active": true }));

    if let Some(ref tab) = activated {
        let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url), false);
        let _ = emit_tab_event(&app_state, "activated", json!({ "tab": tab }));
    } else {
        // 如果没有激活的标签页，隐藏内容窗口
        window_manager::hide_content_window(&app_state.app_handle());
        window_manager::clear_current_url();
    }

    Ok(activated)
}

#[tauri::command]
pub async fn tabs_reload(app_state: State<'_, AppState>, id: String) -> Result<(), String> {
    if let Ok(Some(current_tab)) = TabService::get(app_state.db(), &id).await {
        let tab = Tab::from(current_tab);
        // 强制导航到当前 URL，忽略缓存检查
        let _ = window_manager::present_content_window(&app_state.app_handle(), Some(&tab.url), false);
    }
    Ok(())
}

#[tauri::command]
pub async fn tabs_report_navigation(app_state: State<'_, AppState>, url: String) -> Result<(), String> {
    println!("[NoB] tabs_report_navigation called with: {}", url);
    // 更新数据库中当前激活 Tab 的 URL
    let updated = TabService::update_active_url(app_state.db(), url.clone())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(tab) = updated {
        // 更新缓存
        window_manager::set_current_url(url);
        // 通知前端更新 UI
        let _ = emit_tab_event(&app_state, "updated", json!({ "tab": tab }));
    }
    Ok(())
}

#[tauri::command]
pub async fn tabs_report_title(app_state: State<'_, AppState>, title: String) -> Result<(), String> {
    // 更新数据库中当前激活 Tab 的 Title
    let updated = TabService::update_active_title(app_state.db(), title.clone())
        .await
        .map(|res| res.map(Tab::from))
        .map_err(|e| e.to_string())?;

    if let Some(tab) = updated {
        // 通知前端更新 UI
        let _ = emit_tab_event(&app_state, "updated", json!({ "tab": tab }));
    }
    Ok(())
}

#[tauri::command]
pub async fn tabs_get_current_url(app_state: State<'_, AppState>) -> Result<String, String> {
    use tauri::Manager;
    println!("[NoB] tabs_get_current_url called");
    // 优先尝试从 content window 直接获取
    if let Some(window) = app_state.app_handle().get_webview_window("content") {
        if let Ok(url) = window.url() {
            println!("[NoB] Got URL from window.url(): {}", url);
            return Ok(url.to_string());
        }
    }
    // 降级使用缓存
    let cached = window_manager::get_current_url();
    println!("[NoB] Got URL from cache: {:?}", cached);
    cached.ok_or_else(|| "No active URL".to_string())
}

#[tauri::command]
pub async fn tabs_request_url(app_state: State<'_, AppState>) -> Result<(), String> {
    use tauri::Manager;
    println!("[NoB] tabs_request_url called");
    if let Some(window) = app_state.app_handle().get_webview_window("content") {
        println!("[NoB] Emitting 'get-url' to content window");
        window.emit("get-url", ()).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        println!("[NoB] Content window not found");
        Err("Content window not found".to_string())
    }
}

#[tauri::command]
pub async fn tabs_respond_url(app_state: State<'_, AppState>, url: String) -> Result<(), String> {
    use tauri::Manager;
    println!("[NoB] tabs_respond_url called with: {}", url);
    if let Some(window) = app_state.app_handle().get_webview_window("main") {
        println!("[NoB] Emitting 'return-url' to main window");
        window.emit("return-url", url).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        println!("[NoB] Main window not found");
        Err("Main window not found".to_string())
    }
}

fn emit_tab_event(app_state: &AppState, action: &str, payload: serde_json::Value) -> tauri::Result<()> {
    app_state
        .app_handle()
        .emit("tabs-changed", json!({ "action": action, "data": payload }))
}
