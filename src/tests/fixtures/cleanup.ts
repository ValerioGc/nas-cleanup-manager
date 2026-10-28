import type { CatalogEntry, Cleanup, CleanupRule, Overview, Settings } from '@/types/cleanup';

export const catalog = (): CatalogEntry[] => [
  { id: 'apt-archives', group: 'apt', risky: false },
  { id: 'apt-orphans', group: 'apt', risky: true },
  { id: 'old-kernels', group: 'kernel', risky: true },
  { id: 'journal', group: 'logs', risky: false },
  { id: 'temporary', group: 'temporary', risky: false },
  {
    id: 'cache:nas-file-manager:previews',
    group: 'caches',
    risky: false,
    label: 'Miniature di Explorer',
    app: 'nas-file-manager',
    bytes: 2048,
  },
  {
    id: 'cache:nas-file-manager:trash',
    group: 'trash',
    risky: true,
    label: 'Cestino di Explorer',
    app: 'nas-file-manager',
    bytes: 4096,
  },
];

export const settings = (): Settings => ({ temporaryDays: 7, journalMb: 500, notify: true });

export const cleanup = (overrides: Partial<Cleanup> = {}): Cleanup => ({
  id: 1,
  at: Date.UTC(2026, 9, 10, 8, 30),
  rule: null,
  freed: 3 * 1024 * 1024,
  results: [
    { id: 'temporary', bytes: 3 * 1024 * 1024, entries: [], more: 0 },
    { id: 'journal', bytes: 0, entries: [], more: 0, error: 'journalctl failed' },
  ],
  ...overrides,
});

export const overview = (): Overview => ({
  total: 100 * 1024 ** 3,
  free: 40 * 1024 ** 3,
  categories: [
    { id: 'logs', bytes: 2 * 1024 ** 3 },
    { id: 'packages', bytes: 1024 ** 3 },
    { id: 'caches', bytes: 512 * 1024 ** 2 },
  ],
  disks: [{ name: 'Dati', total: 2 * 1024 ** 4, free: 1024 ** 4 }],
  last: cleanup(),
});

export const rule = (overrides: Partial<CleanupRule> = {}): CleanupRule => ({
  id: 'rule-1',
  name: 'Ogni domenica',
  enabled: true,
  schedule: { frequency: 'weekly', time: '03:00', weekday: 7, day: null, month: null },
  items: ['temporary', 'journal'],
  next: Date.UTC(2026, 9, 18, 1, 0),
  ...overrides,
});
