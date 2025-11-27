import { ArrowLeft, CornerDownLeft, PlusIcon } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";

interface NewTabButtonProps {
  onCreate: (url: string) => void;
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
}

export function NewTabButton({ onCreate, isOpen, onOpenChange }: NewTabButtonProps) {
  const [url, setUrl] = useState("");

  const handleSubmit = () => {
    if (!url.trim()) return;
    
    // Simple URL validation/prefixing
    let targetUrl = url;
    if (!/^https?:\/\//i.test(url)) {
      targetUrl = `https://${url}`;
    }
    
    onCreate(targetUrl);
    setUrl("");
    onOpenChange(false);
  };

  return (
    <>
      <ButtonGroup>
        <Button
          variant={isOpen ? "default" : "outline"}
          size="icon"
          onClick={() => onOpenChange(!isOpen)}
          title={isOpen ? "Cancel" : "New Tab"}
        >
          {isOpen ? <ArrowLeft className="size-4" /> : <PlusIcon className="size-4" />}
        </Button>
      </ButtonGroup>
      <ButtonGroup>
        {isOpen && (
          <InputGroup>
            <InputGroupInput
              className="min-w-[150px]"
              placeholder="Enter URL"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.nativeEvent.isComposing) {
                  handleSubmit();
                }
              }}
              autoFocus
            />
            <InputGroupAddon align="inline-end">
              <InputGroupButton onClick={handleSubmit} size="icon-xs">
                <CornerDownLeft className="size-3" />
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
        )}
      </ButtonGroup>
    </>
  );
}
