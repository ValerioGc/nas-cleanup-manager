import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, expect, it, vi } from 'vitest';
import SchedulePage from '@/components/sections/SchedulePage.vue';
import * as api from '@/services/api';
import { setLocale } from '@/i18n';
import { catalog, rule } from '@/tests/fixtures/cleanup';

vi.mock('@/services/api');

beforeEach(() => {
  setLocale('it');
  vi.resetAllMocks();
  vi.mocked(api.fetchCatalog).mockResolvedValue(catalog());
  vi.mocked(api.fetchRules).mockResolvedValue([rule(), rule({ id: 'rule-2', enabled: false })]);
  vi.mocked(api.saveRule).mockImplementation(async (saved) => saved);
  vi.mocked(api.runRule).mockResolvedValue(null);
  vi.mocked(api.deleteRule).mockResolvedValue(null);
});

async function page() {
  const wrapper = mount(SchedulePage, { attachTo: document.body });
  await flushPromises();
  return wrapper;
}

it('lists the scheduled cleanups with when they run and when next', async () => {
  const wrapper = await page();
  const first = wrapper.get('[data-testid="rule_rule-1"]').text();
  expect(first).toContain('Ogni domenica');
  expect(first).toContain('alle 03:00');
  expect(first).toContain('2 voci');
  expect(wrapper.get('[data-testid="rule_rule-2"]').text()).toContain('Disattivata');
  wrapper.unmount();
});

it('turns a cleanup on or off, runs it now and deletes it after asking', async () => {
  const wrapper = await page();
  await wrapper.get('[data-testid="rule_switch_rule-2"]').trigger('change');
  await flushPromises();
  expect(api.saveRule).toHaveBeenCalledWith(
    expect.objectContaining({ id: 'rule-2', enabled: true }),
  );
  await wrapper.get('[data-testid="rule_run_rule-1"]').trigger('click');
  await flushPromises();
  expect(api.runRule).toHaveBeenCalledWith('rule-1');
  await wrapper.get('[data-testid="rule_delete_rule-1"]').trigger('click');
  expect(wrapper.get('dialog').text()).toContain('La pulizia pianificata Ogni domenica');
  await wrapper.get('[data-testid="rule_delete_confirm"]').trigger('click');
  await flushPromises();
  expect(api.deleteRule).toHaveBeenCalledWith('rule-1');
  vi.mocked(api.runRule).mockRejectedValueOnce(new Error('A cleanup is already running.'));
  await wrapper.get('[data-testid="rule_run_rule-1"]').trigger('click');
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe('A cleanup is already running.');
  wrapper.unmount();
});

it('creates a cleanup with a name, a time and its items, delicate ones only on purpose', async () => {
  const wrapper = await page();
  await wrapper.get('[data-testid="rule_add"]').trigger('click');
  await wrapper.get('[data-testid="rule_save"]').trigger('submit');
  await wrapper.get('form').trigger('submit');
  expect(wrapper.get('dialog [role="alert"]').text()).toBe('È necessario un nome per la pulizia.');
  await wrapper.get('[data-testid="rule_name"]').setValue('  Mensile  ');
  await wrapper.get('[data-testid="rule_frequency"]').setValue('monthly');
  await wrapper.get('[data-testid="rule_day"]').setValue('31');
  await wrapper.get('[data-testid="rule_time"]').setValue('04:15');
  await wrapper.get('[data-testid="item_old-kernels"]').trigger('change');
  expect(wrapper.get('dialog').text()).toContain('Questa pulizia include voci delicate.');
  await wrapper.get('form').trigger('submit');
  await flushPromises();
  expect(api.saveRule).toHaveBeenCalledWith({
    id: '',
    name: 'Mensile',
    enabled: true,
    schedule: { frequency: 'monthly', time: '04:15', weekday: null, day: 31, month: null },
    items: ['apt-archives', 'journal', 'temporary', 'old-kernels'],
  });
  expect(wrapper.find('dialog').exists()).toBe(false);
  wrapper.unmount();
});

it('edits a cleanup, every frequency keeping only its fields', async () => {
  const wrapper = await page();
  await wrapper.get('[data-testid="rule_edit_rule-1"]').trigger('click');
  expect((wrapper.get('[data-testid="rule_name"]').element as HTMLInputElement).value).toBe(
    'Ogni domenica',
  );
  await wrapper.get('[data-testid="rule_frequency"]').setValue('yearly');
  await wrapper.get('[data-testid="rule_month"]').setValue('2');
  await wrapper.get('[data-testid="rule_frequency"]').setValue('weekly');
  await wrapper.get('[data-testid="rule_weekday"]').setValue('3');
  await wrapper.get('[data-testid="rule_frequency"]').setValue('daily');
  await wrapper.get('[data-testid="group_temporary"]').trigger('change');
  await wrapper.get('[data-testid="group_logs"]').trigger('change');
  await wrapper.get('form').trigger('submit');
  expect(wrapper.get('dialog [role="alert"]').text()).toBe('È necessaria almeno una voce.');
  await wrapper.get('[data-testid="item_journal"]').trigger('change');
  vi.mocked(api.saveRule).mockRejectedValueOnce(new Error('The name is 1 to 80 characters.'));
  await wrapper.get('form').trigger('submit');
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe('The name is 1 to 80 characters.');
  expect(api.saveRule).toHaveBeenCalledWith(
    expect.objectContaining({
      id: 'rule-1',
      schedule: { frequency: 'daily', time: '03:00', weekday: null, day: null, month: null },
      items: ['journal'],
    }),
  );
  wrapper.unmount();
});

it('says when there is none, and why they could not be read', async () => {
  vi.mocked(api.fetchRules).mockResolvedValueOnce([]);
  const empty = await page();
  expect(empty.get('[data-testid="rules_empty"] h2').text()).toBe('Nessuna pulizia pianificata');
  empty.unmount();
  vi.mocked(api.fetchRules).mockRejectedValueOnce('down');
  const failed = await page();
  expect(failed.get('[role="alert"]').text()).toBe('Impossibile leggere i dati');
  failed.unmount();
});
