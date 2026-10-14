import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, expect, it, vi } from 'vitest';
import SettingsModal from '@/components/SettingsModal.vue';
import * as api from '@/services/api';
import { setLocale } from '@/i18n';
import { settings } from '@/tests/fixtures/cleanup';

vi.mock('@/services/api');

beforeEach(() => {
  setLocale('it');
  vi.resetAllMocks();
  vi.mocked(api.fetchSettings).mockResolvedValue(settings());
  vi.mocked(api.saveSettings).mockImplementation(async (saved) => saved);
});

function modal() {
  const wrapper = mount(SettingsModal, {
    props: {
      modelValue: true,
      'onUpdate:modelValue': (value: boolean) => wrapper.setProps({ modelValue: value }),
    },
  });
  return wrapper;
}

async function type(wrapper: ReturnType<typeof modal>, testid: string, value: string) {
  const field = wrapper.get(`[data-testid="${testid}"]`);
  await field.setValue(value);
  await field.trigger('change');
}

it('edits the settings and saves them on purpose', async () => {
  const wrapper = modal();
  await flushPromises();
  await type(wrapper, 'settings_days', '14');
  await type(wrapper, 'settings_journal', '256');
  await wrapper.get('[data-testid="settings_notify"]').setValue(false);
  await wrapper.get('form').trigger('submit');
  await flushPromises();
  expect(api.saveSettings).toHaveBeenCalledWith({
    temporaryDays: 14,
    journalMb: 256,
    notify: false,
  });
  expect(wrapper.props('modelValue')).toBe(false);
});

it('keeps the numbers in their bounds, steps them, and says when saving failed', async () => {
  const wrapper = modal();
  await flushPromises();
  await type(wrapper, 'settings_journal', '8');
  expect((wrapper.get('[data-testid="settings_journal"]').element as HTMLInputElement).value).toBe(
    '500',
  );
  await wrapper.get('[data-testid="settings_journal_increase"]').trigger('click');
  await wrapper.get('[data-testid="settings_days_decrease"]').trigger('click');
  vi.mocked(api.saveSettings).mockRejectedValueOnce(new Error('database locked'));
  await wrapper.get('form').trigger('submit');
  await flushPromises();
  expect(api.saveSettings).toHaveBeenCalledWith({ temporaryDays: 6, journalMb: 501, notify: true });
  expect(wrapper.get('[role="alert"]').text()).toBe('database locked');
  expect(wrapper.props('modelValue')).toBe(true);
});

it('says why the settings could not be read', async () => {
  vi.mocked(api.fetchSettings).mockRejectedValueOnce('down');
  const wrapper = modal();
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe('Impossibile leggere i dati');
  expect(wrapper.get('[data-testid="settings_save"]').attributes('disabled')).toBeDefined();
});
