import { useState } from "react";
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

  return (
    <ButtonGroup className="m-1.5 flex items-center" data-tauri-drag-region>
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
