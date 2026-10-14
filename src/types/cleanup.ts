export type SectionCode = 'overview' | 'cleanup' | 'schedule' | 'history';

/** An item a cleanup can take: one of the system, or a cache of an application. */
export interface CatalogEntry {
  id: string;
  group: string;
  risky: boolean;
  /** The name a cache gives itself; the items of the system are named by the catalogue. */
  label?: string;
  app?: string;
  bytes?: number;
}

/** What an item holds (a preview) or freed (a cleanup). */
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

/** A cleanup done: by hand, or by the scheduled cleanup it names. */
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
