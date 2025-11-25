export function reportError(message: string, error: unknown) {
  const details =
    error instanceof Error ? error.message : typeof error === "string" ? error : "未知错误"
  
  // 通过 console.error 记录错误（后端已经通过 NotificationManager 发送通知）
  console.error(`${message}：${details}`, error)
}
