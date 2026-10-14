import { mount } from '@vue/test-utils';
import { expect, it } from 'vitest';
import CommonModal from '@/components/common/CommonModal.vue';

it('closes with its button, Escape or a click beside it', async () => {
  const wrapper = mount(CommonModal, {
    props: {
      title: 'Titolo',
      modelValue: true,
      'onUpdate:modelValue': (value: boolean) => wrapper.setProps({ modelValue: value }),
    },
    slots: { default: 'Contenuto', footer: '<button>Ok</button>' },
    attachTo: document.body,
  });
  expect(wrapper.get('dialog').attributes('aria-label')).toBe('Titolo');
  expect(wrapper.text()).toContain('Contenuto');
  await wrapper.get('dialog').trigger('cancel');
  expect(wrapper.props('modelValue')).toBe(false);
  await wrapper.setProps({ modelValue: true });
  await wrapper.get('.modal_close').trigger('click');
  expect(wrapper.props('modelValue')).toBe(false);
  await wrapper.setProps({ modelValue: true });
  await wrapper.get('dialog').trigger('click');
  expect(wrapper.find('dialog').exists()).toBe(false);
  wrapper.unmount();
});
