// 窗口管理功能
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use tauri::{
    AppHandle, CloseRequestApi, Manager, PhysicalPosition, Position, Runtime,
    WebviewUrl, Window, WebviewWindow, Wry, Size, LogicalSize,
};
use url::Url;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::async_runtime::JoinHandle;

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

static MOVE_TASK: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static MAIN_FOCUSED: AtomicBool = AtomicBool::new(false);
static CONTENT_FOCUSED: AtomicBool = AtomicBool::new(false);
static IS_DRAGGING: AtomicBool = AtomicBool::new(false);
static IS_PINNED: AtomicBool = AtomicBool::new(false);
static FOCUS_CHECK_TASK: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static CURRENT_URL: Mutex<Option<String>> = Mutex::new(None);

/// 设置内容窗口是否固定（不自动隐藏）
pub fn set_content_window_pinned(pinned: bool) {
    IS_PINNED.store(pinned, Ordering::SeqCst);
}

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

/// 清除当前激活的 URL
pub fn clear_current_url() {
    let mut guard = CURRENT_URL.lock().unwrap();
    *guard = None;
}

/// JS 注入脚本：拦截 window.open 和 target=_blank / 跨域链接，改用系统默认浏览器打开。
// 拦截 window.open / 外部链接，改用 Tauri opener 插件从系统浏览器打开。
const EXTERNAL_OPEN_SCRIPT: &str = r#"
(async () => {
  console.log("[NoB] Starting injection script...");

  // --- Helper Functions ---
  const getInvoker = () => {
    const tauri = window.__TAURI__;
    return tauri?.core?.invoke ?? tauri?.invoke;
  };

  const invokeOpen = (url) => {
    const invoker = getInvoker();
    if (!invoker) {
      console.warn("[NoB] __TAURI__ invoke not available");
      return;
    }
    invoker("plugin:opener|open_url", { url, with: null }).catch((e) =>
      console.warn("[NoB] opener open_url rejected", e)
    );
  };

  const isExternal = (href) => {
    try {
      const u = new URL(href, window.location.href);
      return u.origin !== window.location.origin;
    } catch {
      return false;
    }
  };

  const safeInit = async (name, fn) => {
    try {
      await fn();
      console.log(`[NoB] ${name} initialized successfully.`);
    } catch (e) {
      console.error(`[NoB] ${name} initialization failed:`, e);
    }
  };

  // --- Feature Initializers ---

  // 1. Window Open Interceptor
  await safeInit("WindowOpenInterceptor", async () => {
    const originalOpen = window.open;
    window.open = function(url, target, features) {
      if (typeof url === "string") {
        invokeOpen(url);
        return null;
      }
      return originalOpen?.apply(this, [url, target, features]);
    };
  });

  // 2. Navigation Reporter
  await safeInit("NavigationReporter", async () => {
    const reportNavigation = () => {
      const invoker = getInvoker();
      if (invoker) {
        invoker("tabs_report_navigation", { url: window.location.href }).catch(e => 
          console.warn("[NoB] report navigation failed", e)
        );
        // Also report title on navigation as a fallback
        invoker("tabs_report_title", { title: document.title || "Untitled" }).catch(e => 
          console.warn("[NoB] report title failed", e)
        );
      }
    };

    window.addEventListener("popstate", reportNavigation);
    window.addEventListener("hashchange", reportNavigation);

    const originalPushState = history.pushState;
    history.pushState = function(...args) {
      const result = originalPushState.apply(this, args);
      reportNavigation();
      return result;
    };

    const originalReplaceState = history.replaceState;
    history.replaceState = function(...args) {
      const result = originalReplaceState.apply(this, args);
      reportNavigation();
      return result;
    };

    // Initial report
    reportNavigation();
  });

  // 3. Title Reporter
  await safeInit("TitleReporter", async () => {
    const reportTitle = () => {
      const invoker = getInvoker();
      if (invoker) {
        invoker("tabs_report_title", { title: document.title || "Untitled" }).catch(e => 
          console.warn("[NoB] report title failed", e)
        );
      }
    };

    const startObserver = () => {
      const target = document.querySelector('title') || document.head;
      if (target) {
        new MutationObserver(reportTitle).observe(target, { 
          subtree: true, 
          characterData: true, 
          childList: true 
        });
        console.log("[NoB] Title observer started on", target);
        // Report immediately when observer starts
        reportTitle();
      } else {
        console.warn("[NoB] No title or head found to observe");
      }
    };

    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", () => {
        startObserver();
        reportTitle();
      });
    } else {
      startObserver();
      reportTitle();
    }
    
    // Also report on window load to ensure final title is captured
    window.addEventListener("load", reportTitle);
  });

  // 4. External Link Interceptor
  await safeInit("ExternalLinkInterceptor", async () => {
    document.addEventListener("click", (e) => {
      // console.log("[NoB] click event detected");
      const el = e.target;
      if (!(el instanceof Element)) return;
      const anchor = el.closest("a");
      if (!anchor || !anchor.href) return;
      
      const shouldOpenExternally = anchor.target === "_blank" || isExternal(anchor.href);
      
      if (shouldOpenExternally) {
        console.log("[NoB] external link intercepted", anchor.href);
        e.preventDefault();
        invokeOpen(anchor.href);
      }
    }, true);
  });

  // 5. URL Responder
  await safeInit("UrlResponder", async () => {
    const tauri = window.__TAURI__;
    const event = tauri?.event;
    const invoker = getInvoker();

    if (event && invoker) {
      console.log("[NoB] Listening for 'get-url' event");
      await event.listen("get-url", () => {
        console.log("[NoB] Received 'get-url' request, responding with:", window.location.href);
        invoker("tabs_respond_url", { url: window.location.href }).catch(e => 
          console.warn("[NoB] respond url failed", e)
        );
      });
    } else {
      console.warn("[NoB] Tauri event or invoker not available for UrlResponder");
    }
  });

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

/// 调整主窗口大小
pub fn resize_main_window(app: &AppHandle<Wry>, width: f64, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // 避免无效尺寸
        if width <= 0.0 || height <= 0.0 {
            return Ok(());
        }

        // 检查当前尺寸，避免重复调整导致死循环或闪烁
        if let Ok(factor) = window.scale_factor() {
            if let Ok(current_size) = window.inner_size() {
                let current_logical = current_size.to_logical::<f64>(factor);
                if (current_logical.width - width).abs() < 1.0 && (current_logical.height - height).abs() < 1.0 {
                    return Ok(());
                }
            }
        }

        window.set_size(Size::Logical(LogicalSize { width, height })).map_err(|e| e.to_string())?;
        Ok(())
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

    // 仅在位置发生变化时更新，避免不必要的重绘/闪烁
    if let Ok(current_pos) = content_window.outer_position() {
        if current_pos.x == new_pos.x && current_pos.y == new_pos.y {
            return;
        }
    }

    let _ = content_window.set_position(Position::Physical(new_pos));
}

/// 处理主窗口调整大小事件
pub fn on_main_window_resized(app: &AppHandle<Wry>) {
    sync_content_window_position(app);
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

/// 确保内容窗口存在并显示（统一入口）
/// 如果提供了 URL，则更新并导航；如果没有提供，尝试使用缓存的 URL。
/// focus: 是否在显示后聚焦内容窗口
pub fn present_content_window(app: &AppHandle<Wry>, url: Option<&str>, focus: bool) -> Result<(), String> {
    // 1. 确定目标 URL
    let target_url = if let Some(u) = url {
        set_current_url(u.to_string());
        Some(u.to_string())
    } else {
        get_current_url()
    };

    let url_str = match target_url {
        Some(ref u) => u.as_str(),
        None => {
            // 如果没有 URL，确保窗口隐藏
            hide_content_window(app);
            return Ok(());
        }, 
    };

    let parsed = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // 2. 检查窗口是否存在
    if let Some(window) = app.get_webview_window("content") {
        // 复用已有内容窗口
        sync_content_window_position(app);
        
        let is_visible = window.is_visible().unwrap_or(false);

        if !is_visible {
            if focus {
                window.show().map_err(|e| e.to_string())?;
                window.set_focus().map_err(|e| e.to_string())?;
            } else {
                #[cfg(target_os = "macos")]
                {
                    // 记录当前主窗口是否聚焦
                    let was_main_focused = MAIN_FOCUSED.load(Ordering::SeqCst);

                    window.show().map_err(|e| e.to_string())?;
                    
                    // 如果之前主窗口是激活的，强制重新聚焦主窗口
                    if was_main_focused {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            // 延迟一小段时间以确保窗口显示处理完成
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            print!("[NoB] restoring focus to main window\n");
                            if let Some(main_win) = app_handle.get_webview_window("main") {
                                let _ = main_win.set_focus();
                            }
                        });
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    window.show().map_err(|e| e.to_string())?;
                }
            }
        } else if focus {
            // 如果已经显示且需要焦点，则聚焦
            window.set_focus().map_err(|e| e.to_string())?;
        }
        
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
            .focused(focus) // 设置初始焦点状态
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15")
            .initialization_script(EXTERNAL_OPEN_SCRIPT)
            .on_navigation(|url| {
                // 允许所有导航，但可以在这里记录 URL 变化
                // 注意：on_navigation 在 Rust 侧触发，比 JS 更可靠，但可能不包含 pushState
                // 我们主要依赖 JS 注入来处理 SPA，这里作为补充或调试
                println!("[NoB] on_navigation: {}", url);
                true
            })
            .build()
            .map_err(|e| e.to_string())?;
        
        #[cfg(target_os = "macos")]
        {
          use objc2_app_kit::NSColor;
          unsafe {
            if let Ok(raw) = window.ns_window() {
                let ns_window: &NSWindow = &*raw.cast();
                
                ns_window.setOpaque(false);
                let clear = NSColor::clearColor();
                ns_window.setBackgroundColor(Some(&clear));
                
                if let Some(content) = ns_window.contentView() {
                    if let Some(frame) = content.superview() {
                        frame.setWantsLayer(true);
                        if let Some(layer) = frame.layer() {
                            layer.setCornerRadius(12.0);
                            layer.setMasksToBounds(true);
                            layer.setBorderWidth(0.0);
                        }
                    }
                }
            }
          }
        }

        set_window_on_all_workspaces(&window);

        sync_content_window_position(app);
        Ok(())
    }
}

/// 隐藏内容窗口
pub fn hide_content_window(app: &AppHandle<Wry>) {
    if let Some(content_window) = app.get_webview_window("content") {
        // 先移出屏幕以避免磁力吸附
        let _ = content_window.set_position(Position::Physical(PhysicalPosition { x: -10000, y: -10000 }));
        // 然后隐藏
        let _ = content_window.hide();
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
  
  if let Some(window) = app.get_webview_window("main") {
      set_window_on_all_workspaces(&window);
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
             // 注意：这里 focus 设为 false，因为是主窗口获得焦点触发的，不应抢夺焦点
             let _ = present_content_window(app, None, false);
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
            
            // 如果两个窗口都没有焦点，且未固定，则隐藏内容窗口
            if !main_focus && !content_focus {
                if IS_PINNED.load(Ordering::SeqCst) {
                    return;
                }
                if let Some(content) = app_handle.get_webview_window("content") {
                    let _ = content.hide();
                }
            }
        });
        *task_guard = Some(task);
    }
}

#[cfg(target_os = "macos")]
fn set_window_on_all_workspaces(window: &WebviewWindow<Wry>) {
    // 使用 Tauri 内置 API 替代原生调用，避免多线程/UI线程问题
    let _ = window.set_visible_on_all_workspaces(true);
}

#[cfg(target_os = "windows")]
fn set_window_on_all_workspaces(window: &WebviewWindow<Wry>) {
    let _ = window.set_skip_taskbar(true);
}
