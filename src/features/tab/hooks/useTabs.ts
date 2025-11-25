import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

import {
  tabsActivate,
  tabsActivateNext,
  tabsActivatePrevious,
  tabsClose,
  tabsCloseActive,
  tabsCreate,
  tabsList,
  tabsReorder,
  tabsUpdate,
} from "@/features/tab/api/tabs.api";
import type {
  CreateTabInput,
  ReorderTabsInput,
  Tab,
  UpdateTabInput,
} from "@/features/tab/types";

export function useTabs() {
  const queryClient = useQueryClient();

  const tabsQuery = useQuery({
    queryKey: ["tabs"],
    queryFn: tabsList,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["tabs"] });

  const createMutation = useMutation({
    mutationFn: (payload: CreateTabInput) => tabsCreate(payload),
    onSuccess: invalidate,
  });

  const updateMutation = useMutation({
    mutationFn: (payload: UpdateTabInput) => tabsUpdate(payload),
    onSuccess: invalidate,
  });

  const activateMutation = useMutation({
    mutationFn: (id: string) => tabsActivate(id),
    onSuccess: invalidate,
  });

  const closeMutation = useMutation({
    mutationFn: (id: string) => tabsClose(id),
    onSuccess: invalidate,
  });

  const reorderMutation = useMutation({
    mutationFn: (payload: ReorderTabsInput) => tabsReorder(payload),
    onSuccess: invalidate,
  });

  const nextMutation = useMutation({
    mutationFn: () => tabsActivateNext(),
    onSuccess: invalidate,
  });

  const previousMutation = useMutation({
    mutationFn: () => tabsActivatePrevious(),
    onSuccess: invalidate,
  });

  const closeActiveMutation = useMutation({
    mutationFn: () => tabsCloseActive(),
    onSuccess: invalidate,
  });

  // 监听后端触发的 tabs-changed 事件以实时刷新
  useEffect(() => {
    const unlistenPromise = listen("tabs-changed", () => invalidate());
    return () => {
      unlistenPromise.then((un) => un());
    };
  }, [invalidate]);

  return {
    tabs: tabsQuery.data ?? [],
    isLoading: tabsQuery.isLoading,
    error: tabsQuery.error,
    refetch: tabsQuery.refetch,
    createTab: createMutation.mutateAsync,
    updateTab: updateMutation.mutateAsync,
    activateTab: activateMutation.mutateAsync,
    closeTab: closeMutation.mutateAsync,
    reorderTabs: reorderMutation.mutateAsync,
    activateNext: nextMutation.mutateAsync,
    activatePrevious: previousMutation.mutateAsync,
    closeActive: closeActiveMutation.mutateAsync,
    isMutating:
      createMutation.isPending ||
      updateMutation.isPending ||
      activateMutation.isPending ||
      closeMutation.isPending ||
      reorderMutation.isPending ||
      nextMutation.isPending ||
      previousMutation.isPending ||
      closeActiveMutation.isPending,
  };
}
