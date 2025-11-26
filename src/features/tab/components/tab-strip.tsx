import { useEffect, useMemo, useState } from "react";
import { Loader2, Pin, PinOff, Plus, RefreshCw, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea, ScrollBar } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { Tab } from "@/features/tab/types";

type Props = {
  tabs: Tab[];
  isLoading?: boolean;
  isMutating?: boolean;
  onCreate: (url: string) => Promise<void>;
  onActivate: (id: string) => Promise<void>;
  onTogglePin: (id: string, isPinned: boolean) => Promise<void>;
  onClose: (id: string) => Promise<void>;
  onReorder: (orderedIds: string[]) => Promise<void>;
  onRefresh: () => void;
  onNext: () => Promise<void>;
  onPrevious: () => Promise<void>;
  onCloseActive: () => Promise<void>;
};

export function TabStrip({
  tabs,
  isLoading,
  isMutating,
  onCreate,
  onActivate,
  onTogglePin,
  onClose,
  onReorder,
  onRefresh,
  onNext,
  onPrevious,
  onCloseActive,
}: Props) {
  const [url, setUrl] = useState("");
  const [creating, setCreating] = useState(false);
  const [draggingId, setDraggingId] = useState<string | null>(null);

  const activeTab = useMemo(() => tabs.find((t) => t.is_active), [tabs]);
  const orderedIds = useMemo(() => tabs.map((t) => t.id), [tabs]);

  useEffect(() => {
    setCreating(false);
  }, [tabs]);

  const handleTabActivate = async (tab: Tab) => {
    await onActivate(tab.id);
  };

  const handleDrop = (dragId: string, targetId: string) => {
    if (dragId === targetId) return;
    const current = [...orderedIds];
    const from = current.indexOf(dragId);
    const to = current.indexOf(targetId);
    if (from === -1 || to === -1) return;
    current.splice(from, 1);
    current.splice(to, 0, dragId);
    void onReorder(current);
  };

  const handleCreate = async () => {
    if (!url.trim()) return;
    setCreating(true);
    await onCreate(normalizeUrl(url.trim()));
    setUrl("");
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <Input
          placeholder="输入 URL 后按 Enter 创建标签…"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              handleCreate();
            }
          }}
          disabled={creating || isMutating}
        />
        <Button onClick={handleCreate} disabled={creating || isMutating} variant="default">
          {creating ? <Loader2 className="h-4 w-4 animate-spin" /> : <Plus className="h-4 w-4" />}
        </Button>
        <Separator orientation="vertical" className="h-8" />
        <Button variant="ghost" onClick={onRefresh} disabled={isMutating}>
          <RefreshCw className="h-4 w-4" />
        </Button>
      </div>

      <div className="rounded-xl border bg-card/80 p-2 shadow-sm backdrop-blur">
        <div className="flex items-center justify-between gap-2 px-1 pb-2">
          <div className="text-sm font-medium text-muted-foreground">
            {activeTab ? `当前：${activeTab.title}` : "暂无标签"}
          </div>
          <div className="flex items-center gap-1">
            <Button size="sm" variant="ghost" onClick={onPrevious} disabled={isMutating}>
              上一个
            </Button>
            <Button size="sm" variant="ghost" onClick={onNext} disabled={isMutating}>
              下一个
            </Button>
            <Button size="sm" variant="ghost" onClick={onCloseActive} disabled={isMutating}>
              关闭当前
            </Button>
          </div>
        </div>
        <ScrollArea className="w-full whitespace-nowrap">
          <div className="flex items-center gap-2 px-1">
            {isLoading && <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />}
            {tabs.map((tab) => (
              <TabPill
                key={tab.id}
                tab={tab}
                isDragging={draggingId === tab.id}
                onActivate={() => void handleTabActivate(tab)}
                onTogglePin={() => onTogglePin(tab.id, !tab.is_pinned)}
                onClose={() => onClose(tab.id)}
                onDragStart={() => setDraggingId(tab.id)}
                onDragEnd={() => setDraggingId(null)}
                onDropBefore={(targetId) => handleDrop(tab.id, targetId)}
              />
            ))}
          </div>
          <ScrollBar orientation="horizontal" />
        </ScrollArea>
      </div>
    </div>
  );
}

function TabPill({
  tab,
  // isDragging,
  onActivate,
  onTogglePin,
  onClose,
  onDragStart,
  onDragEnd,
  onDropBefore,
}: {
  tab: Tab;
  isDragging: boolean;
  onActivate: () => void;
  onTogglePin: () => void;
  onClose: () => void;
  onDragStart: () => void;
  onDragEnd: () => void;
  onDropBefore: (targetId: string) => void;
}) {
  return (
    <div
      className={`group flex items-center gap-2 rounded-full border px-3 py-2 transition ${
        tab.is_active
          ? "border-primary/60 bg-primary/10 text-primary-foreground"
          : "border-border bg-muted/60 hover:bg-muted"
      }`}
    >
      <button
        onClick={onActivate}
        className="flex items-center gap-2"
        draggable
        onDragStart={onDragStart}
        onDragEnd={onDragEnd}
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          onDropBefore(tab.id);
        }}
      >
        <span className="text-sm font-medium max-w-[12rem] truncate">{tab.title || tab.url}</span>
      </button>
      <div className="flex items-center gap-1 opacity-80">
        <Button
          size="icon"
          variant="ghost"
          className="h-7 w-7"
          onClick={onTogglePin}
          title={tab.is_pinned ? "取消固定" : "固定"}
        >
          {tab.is_pinned ? <Pin className="h-4 w-4" /> : <PinOff className="h-4 w-4" />}
        </Button>
        <Button
          size="icon"
          variant="ghost"
          className="h-7 w-7 hover:text-destructive"
          onClick={onClose}
          title="关闭标签"
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

function normalizeUrl(input: string): string {
  if (input.startsWith("http://") || input.startsWith("https://")) {
    return input;
  }
  return `https://${input}`;
}
