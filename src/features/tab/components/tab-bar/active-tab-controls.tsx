import { Copy, House, RotateCw, X } from "lucide-react";
import { listen } from "@tauri-apps/api/event";

import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import { Label } from "@/components/ui/label";
import { tabsGetCurrentUrl, tabsRequestUrl } from "@/features/tab/api/tabs.api";
import type { Tab } from "@/features/tab/types";

interface ActiveTabControlsProps {
  tab: Tab;
  onHome: (id: string) => void;
  onRefresh: (id: string) => void;
  onClose: (id: string) => void;
}

export function ActiveTabControls({
  tab,
  onHome,
  onRefresh,
  onClose,
}: ActiveTabControlsProps) {
  const getDisplayLabel = () => {
    return tab.title || "Untitled";
  };

  const handleCopy = async () => {
    console.log("[NoB] handleCopy initiated");
    
    // Method 1: Try direct fetch first (fastest)
    try {
      const url = await tabsGetCurrentUrl();
      console.log("[NoB] tabsGetCurrentUrl returned:", url);
      if (url && !url.startsWith("tauri://")) {
        await navigator.clipboard.writeText(url);
        // toast.success("URL copied");
        return;
      }
    } catch (e) {
      console.warn("[NoB] tabsGetCurrentUrl failed, falling back to event flow", e);
    }

    // Method 2: Event-based flow (robust)
    console.log("[NoB] Starting event-based URL request");
    const cleanupPromise = listen<string>("return-url", async (event) => {
      console.log("[NoB] Received 'return-url' event:", event.payload);
      try {
        await navigator.clipboard.writeText(event.payload);
        // toast.success("URL copied");
      } catch (e) {
        console.error("[NoB] Clipboard write failed:", e);
        // toast.error("Failed to write to clipboard");
      }
    });

    try {
      await tabsRequestUrl();
      
      // Cleanup listener after 2 seconds timeout
      setTimeout(async () => {
        const unlisten = await cleanupPromise;
        unlisten();
      }, 2000);
      
    } catch (e) {
      console.error("[NoB] tabsRequestUrl failed:", e);
      // toast.error("Failed to request URL");
      const unlisten = await cleanupPromise;
      unlisten();
    }
  };

  return (
    <ButtonGroup data-tauri-drag-region>
      <ButtonGroup className="bg-gray-400/40 p-0.5 rounded flex items-center">
        <Label
          className="ml-2 min-w-1 select-none max-w-[150px] truncate"
          title={tab.url}
        >
          {getDisplayLabel()}
        </Label>
        <ButtonGroup>
          <Button
            variant="outline"
            size="icon-sm"
            onClick={() => onHome(tab.id)}
            title="Back to Home"
          >
            <House className="size-3" />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            onClick={() => onRefresh(tab.id)}
            title="Refresh"
          >
            <RotateCw className="size-3" />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            onClick={handleCopy}
            title="Copy URL"
          >
            <Copy className="size-3" />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            onClick={() => onClose(tab.id)}
            title="Close Tab"
          >
            <X className="size-3" />
          </Button>
        </ButtonGroup>
      </ButtonGroup>
    </ButtonGroup>
  );
}
