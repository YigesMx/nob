// 窗口管理功能
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use tauri::{
    AppHandle, CloseRequestApi, LogicalPosition, Manager, Runtime, WebviewUrl, Window, Wry,
};
use url::Url;

/// JS 注入脚本：拦截 window.open 和 target=_blank / 跨域链接，改用系统默认浏览器打开。
// 拦截 window.open / 外部链接，改用 Tauri opener 插件从系统浏览器打开。
const EXTERNAL_OPEN_SCRIPT: &str = r#"
(() => {
  console.log("[NoB] injected external-open script");
  const invokeOpen = (url) => {
    const tauri = window.__TAURI__;
    const invoker = tauri?.core?.invoke ?? tauri?.invoke;
    if (!invoker) {
      console.warn("[NoB] __TAURI__ invoke not available in content window");
      return;
    }
    try {
      invoker("plugin:opener|open_url", { url, with: null }).catch((e) =>
        console.warn("[NoB] opener open_url rejected", e)
      );
    } catch (e) {
      console.warn("[NoB] opener open_url failed", e);
    }
  };

  const isExternal = (href) => {
    try {
      const u = new URL(href, window.location.href);
      return u.origin !== window.location.origin;
    } catch {
      return false;
    }
  };

  const originalOpen = window.open;
  window.open = function(url, target, features) {
    if (typeof url === "string") {
      invokeOpen(url);
      return null;
    }
    return originalOpen?.apply(this, [url, target, features]);
  };

  document.addEventListener("click", (e) => {
    console.log("[NoB] click event detected");
    const el = e.target;
    if (!(el instanceof Element)) return;
    const anchor = el.closest("a");
    if (!anchor || !anchor.href) return;
    const shouldOpenExternally =
      anchor.target === "_blank" || isExternal(anchor.href);
    if (shouldOpenExternally) {
      console.log("[NoB] external link intercepted", anchor.href);
      e.preventDefault();
      invokeOpen(anchor.href);
    }
  }, true);
})();
"#;

/// 显示主窗口并设置焦点
/// macOS: 同时显示 Dock 图标
pub fn show_main_window(app: &AppHandle<Wry>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;

        // macOS: 窗口显示时显示 Dock 图标
        #[cfg(target_os = "macos")]
        {
            app.set_activation_policy(tauri::ActivationPolicy::Regular)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

/// 隐藏主窗口
/// macOS: 同时隐藏 Dock 图标
pub fn hide_main_window(app: &AppHandle<Wry>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;

        // macOS: 窗口隐藏时隐藏 Dock 图标
        #[cfg(target_os = "macos")]
        {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

/// 切换主窗口显示/隐藏状态
pub fn toggle_main_window(app: &AppHandle<Wry>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            hide_main_window(app)
        } else {
            show_main_window(app)
        }
    } else {
        Err("Main window not found".to_string())
    }
}

/// 预留：创建内容窗口并加载指定 URL（后续浏览器视图使用）
pub fn open_or_navigate_content_window(app: &AppHandle<Wry>, url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;

    if let Some(window) = app.get_webview_window("content") {
        // 复用已有内容窗口
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        window
            .navigate(parsed.clone())
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        // 新建内容窗口，默认尺寸和位置（后续可根据悬浮条定位）
        tauri::WebviewWindowBuilder::new(app, "content", WebviewUrl::External(parsed))
            .title("NoB 内容")
            .inner_size(1100.0, 780.0)
            .position(120.0, 120.0)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15")
            .initialization_script(EXTERNAL_OPEN_SCRIPT)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 预留：调整内容窗口位置（紧邻悬浮条）
#[allow(dead_code)]
pub fn position_content_window(app: &AppHandle<Wry>, x: f64, y: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("content") {
        window
            .set_position(LogicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 处理窗口关闭请求
///
/// 在桌面平台上，阻止窗口关闭并隐藏窗口
pub fn handle_window_close_request<R: Runtime>(window: &Window<R>, api: &CloseRequestApi) {
    // 仅拦截主窗口，其他窗口允许正常关闭
    if window.label() != "main" {
        return;
    }

    // 阻止窗口关闭，改为隐藏
    api.prevent_close();

    // 隐藏窗口
    let _ = window.hide();

    // macOS: 窗口隐藏时隐藏 Dock 图标
    #[cfg(target_os = "macos")]
    {
        let _ = window
            .app_handle()
            .set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
}
