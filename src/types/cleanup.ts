export type SectionCode = 'overview' | 'cleanup' | 'schedule' | 'history';

/** A system item or an app cache that a cleanup can include. */
export interface CatalogEntry {
  id: string;
  group: string;
  risky: boolean;
  label?: string;
  app?: string;
  bytes?: number;
}

/** Bytes an item holds in a preview or freed in a cleanup. */
export interface ItemResult {
  id: string;
  bytes: number;
  entries: string[];
  more: number;
  error?: string;
}

export interface Settings {
  temporaryDays: number;
  journalMb: number;
  notify: boolean;
}

export type ScheduleFrequency = 'daily' | 'weekly' | 'monthly' | 'yearly';

export interface RuleSchedule {
  frequency: ScheduleFrequency;
  time: string;
  weekday?: number | null;
  day?: number | null;
  month?: number | null;
}

export interface CleanupRule {
  id: string;
  name: string;
  enabled: boolean;
  schedule: RuleSchedule;
  items: string[];
  next?: number | null;
}

/** A completed cleanup. The rule is null when it was started by hand. */
export interface Cleanup {
  id: number;
  at: number;
  rule: string | null;
  freed: number;
  results: ItemResult[];
}

export interface Progress {
  running: boolean;
  rule: string | null;
  item: string | null;
  done: number;
  total: number;
}

export interface Overview {
  total: number;
  free: number;
  categories: { id: string; bytes: number }[];
  disks: { name: string; total: number; free: number }[];
  last: Cleanup | null;
}
