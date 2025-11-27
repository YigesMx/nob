import { Globe } from "lucide-react";

import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import type { Tab } from "@/features/tab/types";

interface TabListProps {
  tabs: Tab[];
  onActivate: (id: string) => void;
}

export function TabList({ tabs, onActivate }: TabListProps) {
  const getFavicon = (tab: Tab) => {
    if (tab.favicon_url) return tab.favicon_url;
    try {
      const url = new URL(tab.url);
      return `https://www.google.com/s2/favicons?domain=${url.hostname}&sz=32`;
    } catch {
      return null;
    }
  };

  return (
    <ButtonGroup>
      {tabs.map((tab) => {
        const favicon = getFavicon(tab);
        return (
          <Button
            key={tab.id}
            variant={tab.is_active ? "default" : "outline"}
            size="icon"
            onClick={() => onActivate(tab.id)}
            title={tab.title}
          >
            {favicon ? (
              <img src={favicon} alt={tab.title} className="size-4 rounded-sm" />
            ) : (
              <Globe className="size-4" />
            )}
          </Button>
        );
      })}
    </ButtonGroup>
  );
}
