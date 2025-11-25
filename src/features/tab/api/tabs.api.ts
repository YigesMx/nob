import { invoke } from "@tauri-apps/api/core";

import type {
  CreateTabInput,
  ReorderTabsInput,
  Tab,
  UpdateTabInput,
} from "@/features/tab/types";

export async function tabsList(): Promise<Tab[]> {
  return invoke<Tab[]>("tabs_list");
}

export async function tabsCreate(payload: CreateTabInput): Promise<Tab> {
  return invoke<Tab>("tabs_create", { payload });
}

export async function tabsUpdate(payload: UpdateTabInput): Promise<Tab | null> {
  return invoke<Tab | null>("tabs_update", { payload });
}

export async function tabsActivate(id: string): Promise<Tab | null> {
  return invoke<Tab | null>("tabs_activate", { id });
}

export async function tabsClose(id: string): Promise<Tab | null> {
  return invoke<Tab | null>("tabs_close", { id });
}

export async function tabsReorder(payload: ReorderTabsInput): Promise<void> {
  return invoke<void>("tabs_reorder", { payload });
}

export async function tabsActivateNext(): Promise<Tab | null> {
  return invoke<Tab | null>("tabs_activate_next");
}

export async function tabsActivatePrevious(): Promise<Tab | null> {
  return invoke<Tab | null>("tabs_activate_previous");
}

export async function tabsCloseActive(): Promise<Tab | null> {
  return invoke<Tab | null>("tabs_close_active");
}
