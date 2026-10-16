import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, expect, it, vi } from 'vitest';
import App from '@/App.vue';
import * as api from '@/services/api';
import { setLocale } from '@/i18n';
import { catalog, overview, settings } from '@/tests/fixtures/cleanup';

vi.mock('@/services/api');

beforeEach(() => {
  setLocale('it');
  vi.mocked(api.fetchOverview).mockResolvedValue(overview());
  vi.mocked(api.fetchCatalog).mockResolvedValue(catalog());
  vi.mocked(api.fetchSelection).mockResolvedValue([]);
  vi.mocked(api.saveSelection).mockResolvedValue({ items: [] });
  vi.mocked(api.fetchSettings).mockResolvedValue(settings());
  vi.mocked(api.fetchProgress).mockResolvedValue({
    running: false,
    rule: null,
    item: null,
    done: 0,
    total: 0,
  });
  vi.mocked(api.fetchRules).mockResolvedValue([]);
  vi.mocked(api.fetchHistory).mockResolvedValue([]);
});

it('opens on the overview and moves between the sections and the settings', async () => {
  const wrapper = mount(App, { attachTo: document.body });
  await flushPromises();
  expect(wrapper.findAll('button.category_button').map((item) => item.text())).toEqual([
    'Panoramica',
    'Pulizia',
    'Pianificazione',
    'Cronologia',
    'Impostazioni',
  ]);
  expect(wrapper.get('h1').text()).toBe('Panoramica');
  expect(wrapper.find('.app_contained').exists()).toBe(false);
  await wrapper.get('[data-testid="go_cleanup"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('h1').text()).toBe('Pulizia');
  // The cleanup scrolls inside the window, its buttons always in view.
  expect(wrapper.find('.app_contained').exists()).toBe(true);
  expect(wrapper.get('[data-testid="section_cleanup"]').attributes('aria-pressed')).toBe('true');
  for (const [section, title] of [
    ['schedule', 'Pianificazione'],
    ['history', 'Cronologia'],
  ]) {
    await wrapper.get(`[data-testid="section_${section}"]`).trigger('click');
    await flushPromises();
    expect(wrapper.get('h1').text()).toBe(title);
  }
  await wrapper.get('[data-testid="open_settings"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('dialog').attributes('aria-label')).toBe('Impostazioni');
  wrapper.unmount();
});
