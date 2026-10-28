import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, expect, it, vi } from 'vitest';
import HistoryPage from '@/components/sections/HistoryPage.vue';
import * as api from '@/services/api';
import { setLocale } from '@/i18n';
import { catalog, cleanup } from '@/tests/fixtures/cleanup';

vi.mock('@/services/api');

beforeEach(() => {
  setLocale('it');
  vi.resetAllMocks();
  vi.mocked(api.fetchCatalog).mockResolvedValue(catalog());
});

it('lists the cleanups with what each item freed or why it was not cleaned', async () => {
  vi.mocked(api.fetchHistory).mockResolvedValue([
    cleanup({ id: 2, rule: 'Ogni domenica' }),
    cleanup({ results: [{ id: 'cache:gone:x', bytes: 5, entries: [], more: 0 }] }),
  ]);
  const wrapper = mount(HistoryPage);
  await flushPromises();
  const scheduled = wrapper.get('[data-testid="history_2"]').text();
  expect(scheduled).toContain('Ogni domenica');
  expect(scheduled).toContain('Liberati 3,0 MB');
  expect(scheduled).toContain('File temporanei');
  expect(scheduled).toContain('Non pulita (journalctl failed)');
  const manual = wrapper.get('[data-testid="history_1"]').text();
  expect(manual).toContain('A mano');
  expect(manual).toContain('cache:gone:x');
});

it('says when there is nothing yet, and why the history could not be read', async () => {
  vi.mocked(api.fetchHistory).mockResolvedValueOnce([]);
  const empty = mount(HistoryPage);
  await flushPromises();
  expect(empty.get('[data-testid="history_empty"] h2').text()).toBe('Nessuna pulizia eseguita');
  vi.mocked(api.fetchHistory).mockRejectedValueOnce(new Error('database locked'));
  const failed = mount(HistoryPage);
  await flushPromises();
  expect(failed.get('[role="alert"]').text()).toBe('database locked');
});
