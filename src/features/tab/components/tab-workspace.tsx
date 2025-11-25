import { useEffect } from "react";
import { toast } from "sonner";

import { Card } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { TabStrip } from "@/features/tab/components/tab-strip";
import { useTabs } from "@/features/tab/hooks/useTabs";

export function TabWorkspace() {
  const {
    tabs,
    isLoading,
    isMutating,
    createTab,
    activateTab,
    updateTab,
    closeTab,
    activateNext,
    activatePrevious,
    closeActive,
    refetch,
    reorderTabs,
  } = useTabs();

  useEffect(() => {
    if (!isLoading && tabs.length === 0) {
      // 提示用户创建首个标签
      toast("没有标签", {
        description: "输入 URL 后回车即可创建第一个标签",
      });
    }
  }, [isLoading, tabs.length]);

  const activeTab = tabs.find((t) => t.is_active);

  return (
    <div className="flex h-dvh flex-col gap-4 bg-gradient-to-br from-background via-background to-muted/40 px-4 py-4">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">NoB · 轻量浏览器</h1>
          <p className="text-sm text-muted-foreground">
            悬浮标签栏 · 单窗口内容区（即将接入内容窗）
          </p>
        </div>
        <div className="text-xs text-muted-foreground">状态：{tabs.length} 个标签</div>
      </header>

      <TabStrip
        tabs={tabs}
        isLoading={isLoading}
        isMutating={isMutating}
        onCreate={(url) => createTab({ url, activate: true })}
        onActivate={(id) => activateTab(id)}
        onTogglePin={(id, isPinned) => updateTab({ id, is_pinned: isPinned })}
        onClose={(id) => closeTab(id)}
        onReorder={(orderedIds) => reorderTabs({ ordered_ids: orderedIds })}
        onRefresh={() => refetch()}
        onNext={() => activateNext()}
        onPrevious={() => activatePrevious()}
        onCloseActive={() => closeActive()}
      />

      <Card className="flex flex-1 flex-col gap-2 overflow-hidden border-muted bg-card/70 p-4 shadow-sm backdrop-blur">
        {activeTab ? (
          <>
            <div className="flex items-center justify-between gap-2">
              <div className="truncate">
                <div className="text-sm font-semibold leading-tight">{activeTab.title}</div>
                <div className="text-xs text-muted-foreground truncate">{activeTab.url}</div>
              </div>
              <div className="text-xs text-muted-foreground">
                最近活跃：{new Date(activeTab.last_opened_at).toLocaleTimeString()}
              </div>
            </div>
            <Separator />
            <div className="flex flex-1 items-center justify-center">
              <div className="text-center text-sm text-muted-foreground leading-relaxed max-w-md">
                内容窗口将在原生 Tauri 窗口中打开（待接入）。当前仅管理标签数据，可通过托盘/WS 控制。
              </div>
            </div>
          </>
        ) : isLoading ? (
          <div className="flex flex-col gap-2">
            <Skeleton className="h-10 w-1/2" />
            <Skeleton className="h-24 w-full" />
          </div>
        ) : (
          <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
            还没有标签，创建一个吧。
          </div>
        )}
      </Card>
    </div>
  );
}
