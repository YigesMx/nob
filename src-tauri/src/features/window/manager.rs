// 窗口管理功能
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use tauri::{
    AppHandle, CloseRequestApi, LogicalPosition, Manager, PhysicalPosition, Position, Runtime,
    WebviewUrl, Window, Wry,
};
use url::Url;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::async_runtime::JoinHandle;

static MOVE_TASK: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static MAIN_FOCUSED: AtomicBool = AtomicBool::new(false);
static CONTENT_FOCUSED: AtomicBool = AtomicBool::new(false);
static IS_DRAGGING: AtomicBool = AtomicBool::new(false);
static FOCUS_CHECK_TASK: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static CURRENT_URL: Mutex<Option<String>> = Mutex::new(None);

/// 设置当前激活的 URL
pub fn set_current_url(url: String) {
    let mut guard = CURRENT_URL.lock().unwrap();
    *guard = Some(url);
}

/// 获取当前激活的 URL
pub fn get_current_url() -> Option<String> {
    let guard = CURRENT_URL.lock().unwrap();
    guard.clone()
}

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

/// 同步内容窗口位置（紧贴主窗口下方）
pub fn sync_content_window_position(app: &AppHandle<Wry>) {
    let main_window = match app.get_webview_window("main") {
        Some(w) => w,
        None => return,
    };
    let content_window = match app.get_webview_window("content") {
        Some(w) => w,
        None => return,
    };

    // 使用物理坐标以确保精确度
    let main_pos = match main_window.outer_position() {
        Ok(p) => p,
        Err(_) => return,
    };
    let main_size = match main_window.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };

    const GAP: i32 = 10; // 间隙像素

    let new_pos = PhysicalPosition {
        x: main_pos.x,
        y: main_pos.y + main_size.height as i32 + GAP,
    };

    let _ = content_window.set_position(Position::Physical(new_pos));
}

/// 处理主窗口移动/调整大小事件（防抖：拖动时隐藏，停止后显示）
pub fn on_main_window_moved(app: &AppHandle<Wry>) {
    let app_handle = app.clone();
    
    // 1. 标记拖动状态并隐藏
    IS_DRAGGING.store(true, Ordering::SeqCst);
    hide_content_window(app);

    // 2. 防抖处理
    let mut task_guard = MOVE_TASK.lock().unwrap();
    
    // 取消上一个任务
    if let Some(task) = task_guard.take() {
        task.abort();
    }

    // 启动新任务
    let task = tauri::async_runtime::spawn(async move {
        // 等待 500ms (或者用户觉得合适的延迟)
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // 拖动结束
        IS_DRAGGING.store(false, Ordering::SeqCst);

        // 重新定位并显示
        sync_content_window_position(&app_handle);
        if let Some(w) = app_handle.get_webview_window("content") {
             let _ = w.show();
             // 保持置顶和焦点可能需要
             // let _ = w.set_focus(); 
        }
    });
    
    *task_guard = Some(task);
}

/// 开始拖动（供前端调用）
pub fn start_dragging(app: &AppHandle<Wry>) {
    IS_DRAGGING.store(true, Ordering::SeqCst);
    hide_content_window(app);
}

/// 确保内容窗口存在并显示（统一入口）
/// 如果提供了 URL，则更新并导航；如果没有提供，尝试使用缓存的 URL。
pub fn present_content_window(app: &AppHandle<Wry>, url: Option<&str>) -> Result<(), String> {
    // 1. 确定目标 URL
    let target_url = if let Some(u) = url {
        set_current_url(u.to_string());
        Some(u.to_string())
    } else {
        get_current_url()
    };

    let url_str = match target_url {
        Some(ref u) => u.as_str(),
        None => return Ok(()), // 没有 URL，无法创建或导航，直接返回
    };

    let parsed = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // 2. 检查窗口是否存在
    if let Some(window) = app.get_webview_window("content") {
        // 复用已有内容窗口
        sync_content_window_position(app);
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        
        // 只有当提供了新的 URL 时才导航，避免刷新
        if url.is_some() {
             window
                .navigate(parsed.clone())
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        // 新建内容窗口
        let window = tauri::WebviewWindowBuilder::new(app, "content", WebviewUrl::External(parsed))
            .title("NoB 内容")
            .inner_size(1100.0, 780.0)
            .position(120.0, 120.0)
            .decorations(false)
            .always_on_top(true)
            .accept_first_mouse(true)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15")
            .initialization_script(EXTERNAL_OPEN_SCRIPT)
            .build()
            .map_err(|e| e.to_string())?;
        
        #[cfg(target_os = "macos")]
        {
          use objc2_app_kit::{NSColor, NSView, NSWindow};
          unsafe {
            let raw = window.ns_window().expect("macOS window pointer");
            let ns_window: &NSWindow = &*raw.cast();
            let content = ns_window.contentView().expect("contentView");
            let frame = content.superview().expect("superview");
            frame.setWantsLayer(true);
            if let Some(layer) = frame.layer() {
              layer.setCornerRadius(12.0);
              layer.setMasksToBounds(true);
              layer.setBorderWidth(0.0);
            }
            ns_window.setOpaque(false);
            let clear = NSColor::clearColor();
            ns_window.setBackgroundColor(Some(&clear));
          }
        }

        sync_content_window_position(app);
        Ok(())
    }
}

/// 兼容旧接口（将被废弃）
pub fn open_or_navigate_content_window(app: &AppHandle<Wry>, url: &str) -> Result<(), String> {
    present_content_window(app, Some(url))
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

/// 隐藏内容窗口
pub fn hide_content_window(app: &AppHandle<Wry>) {
    if let Some(content_window) = app.get_webview_window("content") {
        let _ = content_window.hide();
        // 移出屏幕以避免磁力吸附
        let _ = content_window.set_position(Position::Physical(PhysicalPosition { x: -10000, y: -10000 }));
    }
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

/// 确保应用启动时在 macOS 上不显示 Dock 图标
pub fn configure_startup_behavior(app: &AppHandle<Wry>) {
  #[cfg(target_os = "macos")]
  {
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
  }
}

/// 处理窗口焦点变化
pub fn handle_focus_change(app: &AppHandle<Wry>, window_label: &str, focused: bool) {
    if window_label == "main" {
        MAIN_FOCUSED.store(focused, Ordering::SeqCst);
    } else if window_label == "content" {
        CONTENT_FOCUSED.store(focused, Ordering::SeqCst);
    } else {
        return;
    }

    if focused {
        // 任意窗口获得焦点，取消隐藏任务
        let mut task_guard = FOCUS_CHECK_TASK.lock().unwrap();
        if let Some(task) = task_guard.take() {
            task.abort();
        }
        
        // 如果是主窗口获得焦点，显示内容窗口
        if window_label == "main" {
             // 检查是否正在拖动，如果是则不显示
             if IS_DRAGGING.load(Ordering::SeqCst) {
                 return;
             }

             // 尝试显示内容窗口（如果不存在则尝试创建，前提是有缓存的 URL）
             let _ = present_content_window(app, None);
        }
    } else {
        // 窗口失去焦点，启动延迟检查任务
        let app_handle = app.clone();
        let mut task_guard = FOCUS_CHECK_TASK.lock().unwrap();
        if let Some(task) = task_guard.take() {
            task.abort();
        }
        
        let task = tauri::async_runtime::spawn(async move {
            // 等待 100ms，允许焦点在窗口间切换
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            
            let main_focus = MAIN_FOCUSED.load(Ordering::SeqCst);
            let content_focus = CONTENT_FOCUSED.load(Ordering::SeqCst);
            
            // 如果两个窗口都没有焦点，则隐藏内容窗口
            if !main_focus && !content_focus {
                if let Some(content) = app_handle.get_webview_window("content") {
                    let _ = content.hide();
                }
            }
        });
        *task_guard = Some(task);
    }
}

