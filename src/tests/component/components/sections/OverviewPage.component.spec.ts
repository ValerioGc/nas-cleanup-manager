import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, expect, it, vi } from 'vitest';
import OverviewPage from '@/components/sections/OverviewPage.vue';
import * as api from '@/services/api';
import { setLocale } from '@/i18n';
import { overview } from '@/tests/fixtures/cleanup';

vi.mock('@/services/api');

beforeEach(() => {
  setLocale('it');
  vi.resetAllMocks();
});

it('splits the system disk by category, the system being what the others leave', async () => {
  vi.mocked(api.fetchOverview).mockResolvedValue(overview());
  const wrapper = mount(OverviewPage);
  await flushPromises();
  const category = (id: string) => wrapper.get(`[data-testid="category_${id}"]`).text();
  expect(category('system')).toBe('Sistema 56,5 GB');
  expect(category('logs')).toBe('Log 2,0 GB');
  expect(category('apps')).toBe('App 0 B');
  expect(category('free')).toBe('Libero 40,0 GB');
  expect(wrapper.get('[data-testid="system_disk"]').text()).toContain('40,0 GB liberi su 100,0 GB');
  expect(wrapper.text()).toContain('Dati');
  expect(wrapper.get('[data-testid="last_cleanup"]').text()).toContain('liberati 3,0 MB');
  await wrapper.get('[data-testid="go_cleanup"]').trigger('click');
  expect(wrapper.emitted('cleanup')).toHaveLength(1);
});

it('says when nothing was cleaned and no data disk is connected', async () => {
  vi.mocked(api.fetchOverview).mockResolvedValue({ ...overview(), disks: [], last: null });
  const wrapper = mount(OverviewPage);
  await flushPromises();
  expect(wrapper.text()).toContain('Nessun disco dati collegato.');
  expect(wrapper.get('[data-testid="last_cleanup"]').text()).toBe(
    'Nessuna pulizia eseguita finora.',
  );
});

it('says why the overview is missing and tries again', async () => {
  vi.mocked(api.fetchOverview)
    .mockRejectedValueOnce(new Error('the launcher is not reachable'))
    .mockResolvedValue(overview());
  const wrapper = mount(OverviewPage);
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toContain('the launcher is not reachable');
  await wrapper.get('[role="alert"] button').trigger('click');
  await flushPromises();
  expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  vi.mocked(api.fetchOverview).mockRejectedValueOnce('down');
  const failed = mount(OverviewPage);
  await flushPromises();
  expect(failed.get('[role="alert"]').text()).toContain('Impossibile leggere i dati');
});
