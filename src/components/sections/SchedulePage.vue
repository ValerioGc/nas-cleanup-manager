<script setup lang="ts">

import { onMounted, ref } from 'vue';
import { useI18n } from '@/i18n';
import { deleteRule, fetchCatalog, fetchRules, runRule, saveRule } from '@/services/api';
import { describeSchedule, NasEmptyState, NasLoading } from '@valeriogc/nas-base-app';
import type { CatalogEntry, CleanupRule } from '@/types/cleanup';
import CommonModal from '@/components/common/CommonModal.vue';
import RuleModal from '@/components/schedule/RuleModal.vue';
import addIcon from '@/assets/icons/add.svg?url';
import editIcon from '@/assets/icons/edit.svg?url';
import playIcon from '@/assets/icons/play.svg?url';
import scheduledIcon from '@/assets/icons/scheduled.svg?url';
import trashIcon from '@/assets/icons/trash.svg?url';

const { locale, t } = useI18n();
const rules = ref<CleanupRule[]>([]);
const entries = ref<CatalogEntry[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const editorOpen = ref(false);
const editing = ref<CleanupRule | null>(null);
const saving = ref(false);
const pendingDelete = ref<CleanupRule | null>(null);
const deleteOpen = ref(false);

function reason(failure: unknown) {
  return failure instanceof Error ? failure.message : t.value.loadFailed;
}

function named(text: string, rule: CleanupRule) {
  return text.replace('{name}', rule.name);
}

function next(rule: CleanupRule) {
  if (!rule.enabled) {
    return t.value.schedule.off;
  }
  return rule.next
    ? new Intl.DateTimeFormat(locale.value, { dateStyle: 'short', timeStyle: 'short' }).format(
        rule.next,
      )
    : '-';
}

async function load() {
  try {
    [rules.value, entries.value] = await Promise.all([fetchRules(), fetchCatalog()]);
  } catch (failure) {
    error.value = reason(failure);
  } finally {
    loading.value = false;
  }
}

async function act(action: () => Promise<unknown>) {
  error.value = null;
  try {
    await action();
  } catch (failure) {
    error.value = reason(failure);
  }
  await load();
}

function edit(rule: CleanupRule | null) {
  editing.value = rule;
  editorOpen.value = true;
}

async function save(rule: CleanupRule) {
  saving.value = true;
  await act(async () => {
    await saveRule(rule);
    editorOpen.value = false;
  });
  saving.value = false;
}

function confirmDelete(rule: CleanupRule) {
  pendingDelete.value = rule;
  deleteOpen.value = true;
}

function remove() {
  const rule = pendingDelete.value;
  deleteOpen.value = false;
  if (rule) {
    void act(() => deleteRule(rule.id));
  }
}

onMounted(load);

</script>

<template>
  <section class="schedule_page" :aria-label="t.sections.schedule">
    <header class="page_header page_header_actions">
      <div>
        <h1>{{ t.sections.schedule }}</h1>
        <p>{{ t.schedule.intro }}</p>
      </div>
      <button
        type="button"
        class="app_button app_button--primary"
        data-testid="rule_add"
        @click="edit(null)"
      >
        <img class="app_icon" :src="addIcon" alt="" aria-hidden="true" />
        {{ t.schedule.add }}
      </button>
    </header>
    <p v-if="error" class="app_error" role="alert">{{ error }}</p>
    <NasLoading v-if="loading" :label="t.loading" />
    <NasEmptyState
      v-else-if="!rules.length"
      class="nas_empty_framed"
      :title="t.schedule.empty"
      :description="t.schedule.emptyNote"
      data-testid="rules_empty"
    >
      <template #icon>
        <span class="nas_empty_badge" aria-hidden="true">
          <img class="app_icon" :src="scheduledIcon" alt="" />
        </span>
      </template>
    </NasEmptyState>
    <div v-else class="app_scroll">
      <table class="rule_table">
        <thead>
          <tr>
            <th>{{ t.schedule.name }}</th>
            <th>{{ t.schedule.when }}</th>
            <th>{{ t.schedule.next }}</th>
            <th>{{ t.schedule.enabled }}</th>
            <th><span class="visually_hidden">-</span></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="rule in rules" :key="rule.id" :data-testid="`rule_${rule.id}`">
            <td>
              <strong>{{ rule.name }}</strong>
              <small>{{ t.schedule.items.replace('{count}', String(rule.items.length)) }}</small>
            </td>
            <td>{{ describeSchedule(rule.schedule, locale, t.scheduleSummary) }}</td>
            <td>{{ next(rule) }}</td>
            <td>
              <label class="app_switch" :data-testid="`rule_switch_${rule.id}_switch`">
                <input
                  type="checkbox"
                  role="switch"
                  :name="`rule_switch_${rule.id}`"
                  :checked="rule.enabled"
                  :aria-label="`${t.schedule.enabled}: ${rule.name}`"
                  :data-testid="`rule_switch_${rule.id}`"
                  @change="act(() => saveRule({ ...rule, enabled: !rule.enabled }))"
                />
                <span class="app_switch_track" aria-hidden="true"></span>
              </label>
            </td>
            <td class="rule_actions">
              <button
                type="button"
                class="app_button app_button--icon"
                :aria-label="named(t.schedule.run, rule)"
                :title="named(t.schedule.run, rule)"
                :data-testid="`rule_run_${rule.id}`"
                @click="act(() => runRule(rule.id))"
              >
                <img class="app_icon" :src="playIcon" alt="" aria-hidden="true" />
              </button>
              <button
                type="button"
                class="app_button app_button--icon"
                :aria-label="named(t.schedule.edit, rule)"
                :title="named(t.schedule.edit, rule)"
                :data-testid="`rule_edit_${rule.id}`"
                @click="edit(rule)"
              >
                <img class="app_icon" :src="editIcon" alt="" aria-hidden="true" />
              </button>
              <button
                type="button"
                class="app_button app_button--icon"
                :aria-label="named(t.schedule.delete, rule)"
                :title="named(t.schedule.delete, rule)"
                :data-testid="`rule_delete_${rule.id}`"
                @click="confirmDelete(rule)"
              >
                <img class="app_icon" :src="trashIcon" alt="" aria-hidden="true" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <RuleModal
      v-if="editorOpen"
      v-model="editorOpen"
      :rule="editing"
      :entries="entries"
      :saving="saving"
      @save="save"
    />
    <CommonModal v-model="deleteOpen" :title="t.schedule.deleteTitle">
      <p>{{ pendingDelete ? named(t.schedule.deleteMessage, pendingDelete) : '' }}</p>
      <template #footer>
        <button type="button" class="app_button" @click="deleteOpen = false">
          {{ t.cancel }}
        </button>
        <button
          type="button"
          class="app_button app_button--primary"
          data-testid="rule_delete_confirm"
          @click="remove"
        >
          {{ pendingDelete ? named(t.schedule.delete, pendingDelete) : '' }}
        </button>
      </template>
    </CommonModal>
  </section>
</template>

<style scoped lang="scss">

.schedule_page {
  display: grid;
  gap: 18px;
}

.page_header_actions {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.rule_table {
  th,
  td {
    padding: 10px 8px;
    vertical-align: middle;
  }

  small {
    display: block;
    color: var(--nas-muted);
  }
}

.rule_actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}

.visually_hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
}

</style>
