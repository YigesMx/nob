import { useEffect } from 'react';
import { toast } from 'sonner';
import { listen } from '@tauri-apps/api/event';

interface ToastNotificationPayload {
  message: string;
  level: 'info' | 'success' | 'warning' | 'error';
}

/**
 * NotificationCenter 组件
 * 
 * 统一的通知管理中心，负责监听后端发送的 Tauri Event 并显示通知
 * 
 * 架构设计：
 * - 后端通过 NotificationManager.send_toast() 发送 "toast-notification" 事件
 * - 本组件是唯一监听该事件的地方
 * - 统一管理所有 Sonner Toast 的显示逻辑和配置
 * 
 * 优势：
 * - 关注点分离：业务组件无需关心通知如何显示
 * - 统一配置：所有通知的样式、位置、持续时间等集中管理
 * - 易于维护：切换通知库只需修改此组件
 * - 前后端解耦：后端只需发送标准化的事件
 */
export function NotificationCenter() {
  useEffect(() => {
    console.log('[NotificationCenter] 启动，开始监听 toast-notification 事件');
    
    // 监听后端发送的 toast-notification 事件
    const unlistenPromise = listen<ToastNotificationPayload>('toast-notification', (event) => {
      const { message, level } = event.payload;
      
      console.log(`[NotificationCenter] 收到通知: [${level}] ${message}`);
      
      // 根据级别显示不同类型的 Toast
      switch (level) {
        case 'success':
          toast.success(message, {
            duration: 3000,
          });
          break;
        case 'error':
          toast.error(message, {
            duration: 5000, // 错误提示显示更久
          });
          break;
        case 'warning':
          toast.warning(message, {
            duration: 4000,
          });
          break;
        case 'info':
        default:
          toast.info(message, {
            duration: 3000,
          });
          break;
      }
    });

    // 清理监听器
    return () => {
      console.log('[NotificationCenter] 清理，移除监听器');
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // 此组件无需渲染任何内容
  return null;
}
