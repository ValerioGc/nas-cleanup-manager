import { afterEach, expect, it, vi } from 'vitest';
import * as api from '@/services/api';

const fetcher = vi.fn();
vi.stubGlobal('fetch', fetcher);

function answer(status: number, body: unknown) {
  return {
    ok: status < 300,
    status,
    text: async () => (body === null ? '' : JSON.stringify(body)),
  };
}

afterEach(() => fetcher.mockReset());

it('calls the backend beside the page, with JSON when it sends something', async () => {
  fetcher.mockResolvedValue(answer(200, { items: ['journal'] }));
  expect(await api.fetchSelection()).toEqual(['journal']);
  expect(fetcher).toHaveBeenLastCalledWith('api/selection', {
    method: 'GET',
    headers: undefined,
    body: undefined,
  });
  await api.saveSelection(['journal']);
  expect(fetcher).toHaveBeenLastCalledWith('api/selection', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: '{"items":["journal"]}',
  });
  fetcher.mockResolvedValue(answer(202, null));
  expect(await api.startCleanup(['journal'])).toBeNull();
  for (const call of [
    api.fetchCatalog,
    api.fetchOverview,
    api.fetchProgress,
    api.fetchSettings,
    api.fetchRules,
    api.fetchHistory,
    () => api.previewItems(['journal']),
    () => api.saveSettings({ temporaryDays: 7, journalMb: 500, notify: true }),
    () => api.runRule('a b'),
    () => api.deleteRule('a b'),
  ]) {
    await call();
  }
  expect(fetcher.mock.calls.map(([path]) => path)).toContain('api/rules/a%20b/run');
});

it('creates a rule without an id and changes one with it', async () => {
  fetcher.mockResolvedValue(answer(200, {}));
  const rule = {
    id: '',
    name: 'Notte',
    enabled: true,
    schedule: { frequency: 'daily' as const, time: '03:00' },
    items: ['journal'],
  };
  await api.saveRule(rule);
  expect(fetcher.mock.lastCall?.[0]).toBe('api/rules');
  await api.saveRule({ ...rule, id: 'rule-1' });
  expect(fetcher.mock.lastCall?.[0]).toBe('api/rules/rule-1');
  expect(fetcher.mock.lastCall?.[1].method).toBe('PUT');
});

it('gives the reason of a refusal, or the HTTP status', async () => {
  fetcher.mockResolvedValueOnce(answer(409, { error: 'A cleanup is already running.' }));
  await expect(api.startCleanup(['journal'])).rejects.toThrow('A cleanup is already running.');
  fetcher.mockResolvedValueOnce(answer(502, null));
  await expect(api.fetchOverview()).rejects.toThrow('HTTP 502');
});
