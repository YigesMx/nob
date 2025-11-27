import { GripVertical, Pin } from "lucide-react";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import { Label } from "@/components/ui/label";

export function WindowControls() {
  const [isPinned, setIsPinned] = useState(false);

  const togglePin = async () => {
    const newState = !isPinned;
    setIsPinned(newState);
    await invoke("set_content_window_pinned", { pinned: newState });
  };

  return (
    <>
      <Label data-tauri-drag-region>
        <GripVertical className="size-4 text-muted-foreground" data-tauri-drag-region />
      </Label>
      <ButtonGroup>
        <Button
          variant={isPinned ? "default" : "ghost"}
          size="icon"
          onClick={togglePin}
          title={isPinned ? "Unpin content window" : "Pin content window"}
        >
          <Pin className="size-4" />
        </Button>
      </ButtonGroup>
    </>
  );
}
