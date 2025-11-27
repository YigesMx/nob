import { useEffect } from "react";
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
      // toast removed
    }
  }, [isLoading, tabs.length, createTab]);

  return (
    <div 
      className="flex h-dvh flex-col gap-4 px-4 py-4" 
      data-tauri-drag-region
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
