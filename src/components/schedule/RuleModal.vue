<script setup lang="ts">

import { computed, ref } from 'vue';
import { useI18n } from '@/i18n';
import { monthName, NasSwitch, weekdayName } from '@valeriogc/nas-base-app';
import type { CatalogEntry, CleanupRule, ScheduleFrequency } from '@/types/cleanup';

import CommonModal from '@/components/common/CommonModal.vue';
import ItemTree from '@/components/cleanup/ItemTree.vue';
import warning from '@/assets/icons/warning.svg?url';

const props = defineProps<{ rule: CleanupRule | null; entries: CatalogEntry[]; saving: boolean }>();
const emit = defineEmits<{ save: [rule: CleanupRule] }>();
const open = defineModel<boolean>({ required: true });
const { locale, t } = useI18n();

const frequencies: ScheduleFrequency[] = ['daily', 'weekly', 'monthly', 'yearly'];
const weekdays = [1, 2, 3, 4, 5, 6, 7];
const months = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
const days = Array.from({ length: 31 }, (_, index) => index + 1);

const draft = ref<CleanupRule>(
  props.rule
    ? { ...props.rule, schedule: { ...props.rule.schedule }, items: [...props.rule.items] }
    : {
        id: '',
        name: '',
        enabled: true,
        schedule: { frequency: 'weekly', time: '03:00', weekday: 7, day: null, month: null },
        items: props.entries
          .filter((entry) => !entry.risky && entry.group !== 'caches')
          .map((entry) => entry.id),
      },
);
const problem = ref<string | null>(null);
const risky = computed(() =>
  props.entries.some((entry) => entry.risky && draft.value.items.includes(entry.id)),
);

/** Changes the frequency, keeping the time and clearing the fields it does not use. */
function setFrequency(frequency: ScheduleFrequency) {
  const { time, weekday, day, month } = draft.value.schedule;
  draft.value.schedule = {
    frequency,
    time,
    weekday: frequency === 'weekly' ? (weekday ?? 1) : null,
    day: frequency === 'monthly' || frequency === 'yearly' ? (day ?? 1) : null,
    month: frequency === 'yearly' ? (month ?? 1) : null,
  };
}

function submit() {
  if (!draft.value.name.trim()) {
    problem.value = t.value.schedule.nameRequired;
    return;
  }
  if (!draft.value.items.length) {
    problem.value = t.value.schedule.itemsRequired;
    return;
  }
  problem.value = null;
  emit('save', { ...draft.value, name: draft.value.name.trim() });
}

</script>

<template>
  <CommonModal v-model="open" :title="rule ? t.schedule.editTitle : t.schedule.newTitle" wide>
    <form id="rule_form" class="rule_form" @submit.prevent="submit">
      <label class="field field_name">
        {{ t.schedule.name }}
        <input v-model="draft.name" name="rule_name" maxlength="80" data-testid="rule_name" />
      </label>
      <NasSwitch v-model="draft.enabled" :label="t.schedule.enabled" testid="rule_enabled" />
      <div class="schedule_fields">
        <label class="field">
          {{ t.schedule.frequency }}
          <select
            :value="draft.schedule.frequency"
            name="rule_frequency"
            data-testid="rule_frequency"
            @change="setFrequency(($event.target as HTMLSelectElement).value as ScheduleFrequency)"
          >
            <option v-for="frequency in frequencies" :key="frequency" :value="frequency">
              {{ t.frequencies[frequency] }}
            </option>
          </select>
        </label>
        <label v-if="draft.schedule.frequency === 'weekly'" class="field">
          {{ t.schedule.weekday }}
          <select
            v-model.number="draft.schedule.weekday"
            name="rule_weekday"
            data-testid="rule_weekday"
          >
            <option v-for="weekday in weekdays" :key="weekday" :value="weekday">
              {{ weekdayName(locale, weekday) }}
            </option>
          </select>
        </label>
        <label
          v-if="draft.schedule.frequency === 'monthly' || draft.schedule.frequency === 'yearly'"
          class="field"
        >
          {{ t.schedule.day }}
          <select v-model.number="draft.schedule.day" name="rule_day" data-testid="rule_day">
            <option v-for="day in days" :key="day" :value="day">{{ day }}</option>
          </select>
        </label>
        <label v-if="draft.schedule.frequency === 'yearly'" class="field">
          {{ t.schedule.month }}
          <select v-model.number="draft.schedule.month" name="rule_month" data-testid="rule_month">
            <option v-for="month in months" :key="month" :value="month">
              {{ monthName(locale, month) }}
            </option>
          </select>
        </label>
        <label class="field">
          {{ t.schedule.time }}
          <input
            v-model="draft.schedule.time"
            name="rule_time"
            type="time"
            required
            data-testid="rule_time"
          />
        </label>
      </div>
      <h3>{{ t.schedule.what }}</h3>
      <ItemTree v-model="draft.items" :entries="entries" :label="t.schedule.what" />
      <p v-if="risky" class="alert_warning" role="note">
        <img class="app_icon" :src="warning" alt="" aria-hidden="true" />
        {{ t.schedule.riskyIncluded }}
      </p>
      <p v-if="problem" class="app_error" role="alert">{{ problem }}</p>
    </form>
    <template #footer>
      <button type="button" class="app_button" @click="open = false">{{ t.cancel }}</button>
      <button
        type="submit"
        form="rule_form"
        class="app_button app_button--primary"
        :disabled="saving"
        data-testid="rule_save"
      >
        {{ t.save }}
      </button>
    </template>
  </CommonModal>
</template>

<style scoped lang="scss">

.rule_form {
  display: grid;
  gap: 14px;

  h3 {
    margin: 4px 0 0;
    font-size: 1rem;
  }
}

.schedule_fields {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.field {
  display: grid;
  gap: 4px;
  color: var(--nas-muted);

  input,
  select {
    min-height: 38px;
    padding: 6px 10px;
    border: 1px solid var(--nas-border);
    border-radius: 8px;
    background: var(--nas-surface);
    color: var(--nas-foreground);
  }
}

.field_name {
  max-width: 360px;
}

</style>
