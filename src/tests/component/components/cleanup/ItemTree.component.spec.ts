import { mount } from '@vue/test-utils';
import { expect, it } from 'vitest';
import ItemTree from '@/components/cleanup/ItemTree.vue';
import { setLocale } from '@/i18n';
import { catalog } from '@/tests/fixtures/cleanup';

function tree(selected: string[]) {
  setLocale('it');
  const wrapper = mount(ItemTree, {
    props: {
      entries: catalog(),
      label: 'Voci',
      modelValue: selected,
      'onUpdate:modelValue': (value: string[]) => wrapper.setProps({ modelValue: value }),
      active: null,
      'onUpdate:active': (value: string | null) => wrapper.setProps({ active: value }),
      results: { 'apt-archives': { id: 'apt-archives', bytes: 1536, entries: [], more: 0 } },
    },
  });
  return wrapper;
}

const box = (wrapper: ReturnType<typeof tree>, id: string) =>
  wrapper.get(`[data-testid="${id}"]`).element as HTMLInputElement;

it('takes or leaves a whole group, and shows half of it when only some is taken', async () => {
  const wrapper = tree(['apt-archives']);
  expect(wrapper.findAll('section').map((section) => section.get('strong').text())).toEqual([
    'Pacchetti Debian (APT)',
    'Kernel',
    'Log',
    'File temporanei',
    'Cache delle app',
    'Cestini',
  ]);
  expect(box(wrapper, 'group_apt').indeterminate).toBe(true);
  expect(wrapper.get('section').text()).toContain('1 di 2');
  await wrapper.get('[data-testid="group_apt"]').trigger('change');
  expect(wrapper.props('modelValue')).toEqual(['apt-archives', 'apt-orphans']);
  expect(box(wrapper, 'group_apt').checked).toBe(true);
  await wrapper.get('[data-testid="group_apt"]').trigger('change');
  expect(wrapper.props('modelValue')).toEqual([]);
});

it('says why a delicate item is delicate once it is taken, and shows its measured size', async () => {
  const wrapper = tree([]);
  expect(wrapper.find('[data-testid="risky_old-kernels"]').exists()).toBe(false);
  expect(wrapper.get('[data-testid="show_old-kernels"] img').attributes('alt')).toBe(
    'Voce delicata',
  );
  await wrapper.get('[data-testid="item_old-kernels"]').trigger('change');
  expect(wrapper.get('[data-testid="risky_old-kernels"]').text()).toContain('kernel di riserva');
  expect(wrapper.get('[data-testid="risky_old-kernels"] strong').text()).toBe('Attenzione');
  await wrapper.get('[data-testid="item_old-kernels"]').trigger('change');
  expect(wrapper.props('modelValue')).toEqual([]);
  await wrapper.get('[data-testid="item_cache:nas-file-manager:trash"]').trigger('change');
  expect(wrapper.get('[data-testid="risky_cache:nas-file-manager:trash"]').text()).toContain(
    'Lo svuotamento li elimina definitivamente',
  );
  expect(wrapper.text()).toContain('1,5 KB');
  expect(wrapper.text()).toContain('Miniature di Explorer');
});

it('opens and closes a group, and shows an item when it is chosen', async () => {
  const wrapper = tree([]);
  const toggle = wrapper.get('[aria-label="Chiudi Kernel"]');
  await toggle.trigger('click');
  expect(wrapper.find('[data-testid="item_old-kernels"]').exists()).toBe(false);
  await wrapper.get('[aria-label="Apri Kernel"]').trigger('click');
  expect(wrapper.find('[data-testid="item_old-kernels"]').exists()).toBe(true);
  await wrapper.get('[data-testid="show_journal"]').trigger('click');
  expect(wrapper.props('active')).toBe('journal');
  expect(wrapper.get('[data-testid="show_journal"]').attributes('aria-pressed')).toBe('true');
});
