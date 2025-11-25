export type Tab = {
  id: string;
  title: string;
  url: string;
  favicon_url?: string | null;
  is_pinned: boolean;
  is_active: boolean;
  sort_order: number;
  last_opened_at: string;
  created_at: string;
  updated_at: string;
};

export type CreateTabInput = {
  url: string;
  title?: string;
  favicon_url?: string;
  is_pinned?: boolean;
  activate?: boolean;
};

export type UpdateTabInput = {
  id: string;
  title?: string;
  url?: string;
  favicon_url?: string;
  is_pinned?: boolean;
};

export type ReorderTabsInput = {
  ordered_ids: string[];
};
