import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import CleanupPage from '@/components/sections/CleanupPage.vue';
import * as api from '@/services/api';
import { setLocale } from '@/i18n';
import { catalog, cleanup, settings } from '@/tests/fixtures/cleanup';
import type { Progress } from '@/types/cleanup';

vi.mock('@/services/api');

const idle: Progress = { running: false, rule: null, item: null, done: 0, total: 0 };

beforeEach(() => {
  vi.useFakeTimers();
  setLocale('it');
  vi.mocked(api.fetchCatalog).mockResolvedValue(catalog());
  vi.mocked(api.fetchSelection).mockResolvedValue(['temporary', 'journal', 'gone']);
  vi.mocked(api.fetchSettings).mockResolvedValue(settings());
  vi.mocked(api.fetchProgress).mockResolvedValue(idle);
  vi.mocked(api.saveSelection).mockResolvedValue({ items: [] });
  vi.mocked(api.previewItems).mockResolvedValue([
    { id: 'temporary', bytes: 2048, entries: ['/tmp/old.txt'], more: 3 },
    { id: 'journal', bytes: 0, entries: [], more: 0, error: 'journalctl failed' },
  ]);
  vi.mocked(api.startCleanup).mockResolvedValue(null);
  vi.mocked(api.fetchHistory).mockResolvedValue([cleanup()]);
});

afterEach(() => {
  vi.useRealTimers();
  vi.resetAllMocks();
});

async function page() {
  const wrapper = mount(CleanupPage, { attachTo: document.body });
  await flushPromises();
  return wrapper;
}

it('starts from the kept choice, without the items gone since, and keeps every change', async () => {
  const wrapper = await page();
  expect((wrapper.get('[data-testid="item_temporary"]').element as HTMLInputElement).checked).toBe(
    true,
  );
  expect(wrapper.get('[data-testid="item_detail"]').text()).toContain('Pacchetti scaricati');
  await wrapper.get('[data-testid="item_apt-archives"]').trigger('change');
  await flushPromises();
  expect(api.saveSelection).toHaveBeenCalledWith(['temporary', 'journal', 'apt-archives']);
  wrapper.unmount();
});

it('measures the choice and shows what each item would free', async () => {
  const wrapper = await page();
  await wrapper.get('[data-testid="show_temporary"]').trigger('click');
  expect(wrapper.text()).toContain('Valore disponibile dopo l’anteprima.');
  expect(wrapper.text()).toContain('File non modificati da più di 7 giorni');
  await wrapper.get('[data-testid="cleanup_preview"]').trigger('click');
  await flushPromises();
  expect(api.previewItems).toHaveBeenCalledWith(['temporary', 'journal']);
  expect(wrapper.get('[data-testid="detail_size"]').text()).toBe('2,0 KB');
  expect(wrapper.text()).toContain('/tmp/old.txt');
  expect(wrapper.text()).toContain('e altri 3');
  expect(wrapper.get('[data-testid="cleanup_total"]').text()).toBe('Totale liberabile 2,0 KB');
  await wrapper.get('[data-testid="show_journal"]').trigger('click');
  expect(wrapper.text()).toContain('Non misurabile (journalctl failed)');
  expect(wrapper.text()).toContain('(500,0 MB)');
  wrapper.unmount();
});

it('asks before cleaning, follows the cleanup and says what it freed', async () => {
  vi.mocked(api.fetchSelection).mockResolvedValue(['temporary', 'old-kernels']);
  vi.mocked(api.fetchProgress)
    .mockResolvedValueOnce(idle)
    .mockResolvedValueOnce({ running: true, rule: null, item: 'temporary', done: 1, total: 2 })
    .mockResolvedValue(idle);
  const wrapper = await page();
  await wrapper.get('[data-testid="cleanup_start"]').trigger('click');
  const dialog = wrapper.get('dialog');
  expect(dialog.text()).toContain('Verranno pulite 2 voci.');
  expect(dialog.text()).toContain('Kernel vecchi');
  await wrapper.get('[data-testid="cleanup_confirm"]').trigger('click');
  await flushPromises();
  expect(api.startCleanup).toHaveBeenCalledWith(['temporary', 'old-kernels']);
  expect(wrapper.get('[data-testid="cleanup_progress"]').text()).toContain(
    'Pulizia in corso, 1 di 2',
  );
  expect(wrapper.get('[data-testid="cleanup_start"]').attributes('disabled')).toBeDefined();
  await vi.advanceTimersByTimeAsync(1000);
  await flushPromises();
  expect(wrapper.get('[data-testid="cleanup_outcome"]').text()).toBe(
    'Pulizia completata, liberati 3,0 MB. 1 voci non pulite.',
  );
  wrapper.unmount();
});

it('says a cleanup without errors ended well, and what went wrong otherwise', async () => {
  vi.mocked(api.fetchHistory).mockResolvedValueOnce([
    cleanup({ results: [{ id: 'temporary', bytes: 10, entries: [], more: 0 }] }),
  ]);
  vi.mocked(api.fetchProgress)
    .mockResolvedValueOnce({ running: true, rule: 'Notte', item: null, done: 0, total: 1 })
    .mockResolvedValue(idle);
  const wrapper = await page();
  await vi.advanceTimersByTimeAsync(1000);
  await flushPromises();
  expect(wrapper.get('[data-testid="cleanup_outcome"]').text()).toBe(
    'Pulizia completata, liberati 3,0 MB.',
  );
  vi.mocked(api.previewItems).mockRejectedValueOnce(new Error('launcher unavailable'));
  await wrapper.get('[data-testid="cleanup_preview"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe('launcher unavailable');
  vi.mocked(api.startCleanup).mockRejectedValueOnce(new Error('busy'));
  await wrapper.get('[data-testid="cleanup_start"]').trigger('click');
  await wrapper.get('[data-testid="cleanup_confirm"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe('busy');
  wrapper.unmount();
});

it('says why the items could not be read, and waits for a choice', async () => {
  vi.mocked(api.fetchCatalog).mockRejectedValueOnce('down');
  const failed = await page();
  expect(failed.get('[role="alert"]').text()).toBe('Impossibile leggere i dati');
  failed.unmount();
  vi.mocked(api.fetchSelection).mockResolvedValue([]);
  const empty = await page();
  expect(empty.text()).toContain('È necessaria almeno una voce.');
  expect(empty.get('[data-testid="cleanup_preview"]').attributes('disabled')).toBeDefined();
  empty.unmount();
});
