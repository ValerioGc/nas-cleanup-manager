import type {
  CatalogEntry,
  Cleanup,
  CleanupRule,
  ItemResult,
  Overview,
  Progress,
  Settings,
} from '@/types/cleanup';

/** A refusal of the backend, with the reason it gave. */
export class ApiFailure extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

/** One request to the backend of the application, beside the page. */
async function call<T>(path: string, init?: { method: string; body?: unknown }): Promise<T> {
  const response = await fetch(`api/${path}`, {
    method: init?.method ?? 'GET',
    headers: init?.body === undefined ? undefined : { 'Content-Type': 'application/json' },
    body: init?.body === undefined ? undefined : JSON.stringify(init.body),
  });
  const text = await response.text();
  const body: unknown = text ? JSON.parse(text) : null;
  if (!response.ok) {
    const reason =
      body && typeof body === 'object' && 'error' in body && typeof body.error === 'string'
        ? body.error
        : `HTTP ${response.status}`;
    throw new ApiFailure(reason, response.status);
  }
  return body as T;
}

export const fetchCatalog = () => call<CatalogEntry[]>('catalog');
export const fetchOverview = () => call<Overview>('overview');
export const fetchSelection = () => call<{ items: string[] }>('selection').then((s) => s.items);
export const saveSelection = (items: string[]) =>
  call<{ items: string[] }>('selection', { method: 'PUT', body: { items } });
export const previewItems = (items: string[]) =>
  call<ItemResult[]>('preview', { method: 'POST', body: { items } });
export const startCleanup = (items: string[]) =>
  call<null>('clean', { method: 'POST', body: { items } });
export const fetchProgress = () => call<Progress>('progress');
export const fetchSettings = () => call<Settings>('settings');
export const saveSettings = (settings: Settings) =>
  call<Settings>('settings', { method: 'PUT', body: settings });
export const fetchRules = () => call<CleanupRule[]>('rules');
export const saveRule = (rule: CleanupRule) =>
  rule.id
    ? call<CleanupRule>(`rules/${encodeURIComponent(rule.id)}`, { method: 'PUT', body: rule })
    : call<CleanupRule>('rules', { method: 'POST', body: rule });
export const deleteRule = (id: string) =>
  call<null>(`rules/${encodeURIComponent(id)}`, { method: 'DELETE' });
export const runRule = (id: string) =>
  call<null>(`rules/${encodeURIComponent(id)}/run`, { method: 'POST' });
export const fetchHistory = () => call<Cleanup[]>('history');
