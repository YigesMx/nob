import { useEffect } from "react";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";

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

  const handleDragStart = () => {
    invoke("window_drag_start").catch(console.error);
  };

  return (
    <div 
      className="flex h-dvh flex-col gap-4 px-4 py-4" 
      data-tauri-drag-region
      onMouseDown={handleDragStart}
    >
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
    </div>
  );
}
