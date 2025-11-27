import { useState, useRef, useLayoutEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ButtonGroup } from "@/components/ui/button-group";
import { useTabs } from "@/features/tab/hooks/useTabs";
import { ActiveTabControls } from "./active-tab-controls";
import { NewTabButton } from "./new-tab-button";
import { TabList } from "./tab-list";
import { WindowControls } from "./window-controls";

export function TabBar() {
  const {
    tabs,
    createTab,
    activateTab,
    closeTab,
    updateTab,
    reloadTab,
  } = useTabs();

  const [isAddingTab, setIsAddingTab] = useState(false);

  const activeTab = tabs.find((t) => t.is_active);

  const handleCreate = (url: string) => {
    createTab({ url, activate: true });
  };

  const handleActivate = (id: string) => {
    activateTab(id);
  };

  const handleClose = (id: string) => {
    closeTab(id);
  };

  const handleHome = (id: string) => {
    const tab = tabs.find((t) => t.id === id);
    if (tab && tab.initial_url) {
      updateTab({ id, url: tab.initial_url }).then(() => {
        reloadTab(id);
      });
    }
  };

  const handleRefresh = (id: string) => {
    reloadTab(id);
  };

  const containerRef = useRef<HTMLDivElement>(null);
  const resizeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useLayoutEffect(() => {
    if (!containerRef.current) return;

    const updateSize = () => {
      if (containerRef.current) {
        const { offsetWidth, offsetHeight } = containerRef.current;
        const style = window.getComputedStyle(containerRef.current);
        const marginLeft = parseFloat(style.marginLeft) || 0;
        const marginRight = parseFloat(style.marginRight) || 0;
        const marginTop = parseFloat(style.marginTop) || 0;
        const marginBottom = parseFloat(style.marginBottom) || 0;
        
        const width = offsetWidth + marginLeft + marginRight;
        const height = offsetHeight + marginTop + marginBottom;

        if (width <= 0 || height <= 0) return;

        // Debounce resize calls
        if (resizeTimeoutRef.current) {
          clearTimeout(resizeTimeoutRef.current);
        }

        resizeTimeoutRef.current = setTimeout(() => {
          invoke("resize_main_window", { width, height }).catch(console.error);
        }, 50);
      }
    };

    const observer = new ResizeObserver(updateSize);
    observer.observe(containerRef.current);
    
    // Initial call
    updateSize();

    return () => {
      observer.disconnect();
      if (resizeTimeoutRef.current) {
        clearTimeout(resizeTimeoutRef.current);
      }
    };
  }, []);

  return (
    <ButtonGroup ref={containerRef} className="m-1.5 flex items-center" data-tauri-drag-region>
      <WindowControls />
      
      <TabList tabs={tabs} onActivate={handleActivate} />
      
      <NewTabButton 
        onCreate={handleCreate} 
        isOpen={isAddingTab}
        onOpenChange={setIsAddingTab}
      />
      
      {activeTab && !isAddingTab && (
        <ActiveTabControls
          tab={activeTab}
          onHome={handleHome}
          onRefresh={handleRefresh}
          onClose={handleClose}
        />
      )}
    </ButtonGroup>
  );
}
