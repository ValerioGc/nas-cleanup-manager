<script setup lang="ts">

import { NasModal } from '@valeriogc/nas-base-app';
import { useI18n } from '@/i18n';
import close from '@/assets/icons/close.svg?url';

defineProps<{ title: string; wide?: boolean }>();
const open = defineModel<boolean>({ required: true });
const { t } = useI18n();

</script>

<template>
  <NasModal v-if="open" :label="title" @close="open = false">
    <section :class="['modal', { modal_wide: wide }]">
      <header class="modal_heading">
        <h2>{{ title }}</h2>
        <button
          type="button"
          class="app_button app_button--icon modal_close"
          :aria-label="t.close"
          :title="t.close"
          @click="open = false"
        >
          <img class="app_icon" :src="close" alt="" aria-hidden="true" />
        </button>
      </header>
      <div class="modal_body">
        <slot></slot>
      </div>
      <footer v-if="$slots.footer" class="modal_footer app_toolbar">
        <slot name="footer"></slot>
      </footer>
    </section>
  </NasModal>
</template>

<style scoped lang="scss">

.modal {
  display: flex;
  flex-direction: column;
  width: min(560px, 100%);
  max-height: calc(100vh - 40px);
  overflow: hidden;
  border: 1px solid var(--nas-border);
  border-radius: 14px;
  background: var(--nas-surface);
}

.modal_wide {
  width: min(820px, 100%);
}

.modal_heading {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 18px 24px 0;

  h2 {
    margin: 0;
    font-size: 1.15rem;
  }
}

.modal_close {
  border-color: transparent;
}

.modal_body {
  min-height: 0;
  padding: 16px 24px 20px;
  overflow: auto;
}

.modal_footer {
  flex: none;
  justify-content: flex-end;
  padding: 0 24px 20px;
}

</style>
