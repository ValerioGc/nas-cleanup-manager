<script setup lang="ts">

import { ref, useId, watch } from 'vue';
import { NasNumberStepper, NasSwitch } from '@valeriogc/nas-base-app';
import { useI18n } from '@/i18n';
import { fetchSettings, saveSettings } from '@/services/api';
import type { Settings } from '@/types/cleanup';

import CommonModal from '@/components/common/CommonModal.vue';

const open = defineModel<boolean>({ required: true });
const { t } = useI18n();
const draft = ref<Settings | null>(null);
const error = ref<string | null>(null);
const saving = ref(false);
const daysId = useId();
const journalId = useId();

function reason(failure: unknown) {
  return failure instanceof Error ? failure.message : t.value.loadFailed;
}

watch(
  open,
  async (shown) => {
    if (!shown) {
      return;
    }
    error.value = null;
    try {
      draft.value = await fetchSettings();
    } catch (error_) {
      error.value = reason(error_);
    }
  },
  { immediate: true },
);

async function submit() {
  if (!draft.value) {
    return;
  }
  saving.value = true;
  try {
    await saveSettings(draft.value);
    open.value = false;
  } catch (error_) {
    error.value = reason(error_);
  } finally {
    saving.value = false;
  }
}

</script>

<template>
  <CommonModal v-model="open" :title="t.settings">
    <form v-if="draft" id="settings_form" class="settings_form" @submit.prevent="submit">
      <div class="field">
        <label class="field_label" :for="daysId">{{ t.settingsModal.temporaryDays }}</label>

        <NasNumberStepper
          :id="daysId"
          v-model="draft.temporaryDays"
          class="field_number"
          :min="1"
          :max="3650"
          :increase-label="t.increase"
          :decrease-label="t.decrease"
          testid="settings_days"
        />
        <small>{{ t.settingsModal.temporaryDaysHint }}</small>
      </div>

      <div class="field">
        <label class="field_label" :for="journalId">{{ t.settingsModal.journalMb }}</label>
        <NasNumberStepper
          :id="journalId"
          v-model="draft.journalMb"
          class="field_number"
          :min="32"
          :max="102400"
          :increase-label="t.increase"
          :decrease-label="t.decrease"
          testid="settings_journal"
        />
        <small>{{ t.settingsModal.journalHint }}</small>
      </div>
      <div class="field">
        <NasSwitch
          v-model="draft.notify"
          :label="t.settingsModal.notify"
          testid="settings_notify"
        />
        <small>{{ t.settingsModal.notifyHint }}</small>
      </div>
    </form>

    <p v-if="error" class="app_error" role="alert">{{ error }}</p>
    
    <template #footer>
      <button type="button" class="app_button" @click="open = false">{{ t.cancel }}</button>
      <button
        type="submit"
        form="settings_form"
        class="app_button app_button--primary"
        :disabled="!draft || saving"
        data-testid="settings_save"
      >
        {{ t.save }}
      </button>
    </template>
  </CommonModal>
</template>

<style scoped lang="scss">

.settings_form {
  display: grid;
  gap: 20px;
}

.field {
  display: grid;
  justify-items: start;
  gap: 6px;
}

.field_label {
  font-weight: 600;
}

.field_number {
  width: 160px;
}

small {
  display: block;
  color: var(--nas-muted);
}

</style>
