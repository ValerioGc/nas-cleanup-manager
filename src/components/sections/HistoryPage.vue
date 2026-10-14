<script setup lang="ts">

import { onMounted, ref } from 'vue';
import { useI18n } from '@/i18n';
import { fetchCatalog, fetchHistory } from '@/services/api';
import { formatBytes, NasEmptyState, NasLoading } from '@valeriogc/nas-base-app';
import { itemName } from '@/services/items';
import type { CatalogEntry, Cleanup } from '@/types/cleanup';
import failedIcon from '@/assets/icons/status-failed.svg?url';
import historyIcon from '@/assets/icons/history.svg?url';
import okIcon from '@/assets/icons/status-ok.svg?url';

const { locale, t } = useI18n();
const cleanups = ref<Cleanup[]>([]);
const entries = ref<CatalogEntry[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

function when(at: number) {
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'medium', timeStyle: 'short' }).format(
    at,
  );
}

/** The name of an item of a past cleanup: a cache gone since keeps its id. */
function named(id: string) {
  return itemName(t.value, entries.value.find((entry) => entry.id === id) ?? { id });
}

onMounted(async () => {
  try {
    [cleanups.value, entries.value] = await Promise.all([fetchHistory(), fetchCatalog()]);
  } catch (failure) {
    error.value = failure instanceof Error ? failure.message : t.value.loadFailed;
  } finally {
    loading.value = false;
  }
});

</script>

<template>
  <section class="history_page" :aria-label="t.sections.history">
    <header class="page_header">
      <h1>{{ t.sections.history }}</h1>
    </header>
    <NasLoading v-if="loading" :label="t.loading" />
    <p v-else-if="error" class="app_error" role="alert">{{ error }}</p>
    <NasEmptyState
      v-else-if="!cleanups.length"
      class="nas_empty_framed"
      :title="t.history.empty"
      :description="t.history.emptyNote"
      data-testid="history_empty"
    >
      <template #icon>
        <span class="nas_empty_badge" aria-hidden="true">
          <img class="app_icon" :src="historyIcon" alt="" />
        </span>
      </template>
    </NasEmptyState>
    <ol v-else class="history_list">
      <li
        v-for="cleanup in cleanups"
        :key="cleanup.id"
        class="app_panel history_entry"
        :data-testid="`history_${cleanup.id}`"
      >
        <header class="history_heading">
          <strong>{{ when(cleanup.at) }}</strong>
          <span class="muted">{{ cleanup.rule ?? t.history.manual }}</span>
          <span class="history_freed">
            {{ t.history.freed.replace('{size}', formatBytes(cleanup.freed, locale)) }}
          </span>
        </header>
        <ul class="history_items">
          <li v-for="result in cleanup.results" :key="result.id">
            <img
              class="app_icon"
              :src="result.error ? failedIcon : okIcon"
              alt=""
              aria-hidden="true"
            />
            <span>{{ named(result.id) }}</span>
            <span v-if="result.error" class="app_error">
              {{ t.history.failed.replace('{reason}', result.error) }}
            </span>
            <span v-else class="muted">{{ formatBytes(result.bytes, locale) }}</span>
          </li>
        </ul>
      </li>
    </ol>
  </section>
</template>

<style scoped lang="scss">

.history_page {
  display: grid;
  gap: 18px;
}

.history_list {
  @extend %reset_list;
  display: grid;
  gap: 12px;
}

.history_entry {
  padding: 14px 18px;
}

.history_heading {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px 14px;
}

.history_freed {
  margin-left: auto;
  font-weight: 600;
}

.history_items {
  @extend %reset_list;
  display: grid;
  gap: 4px;
  margin-top: 10px;

  li {
    --app-icon-size: 16px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
}

</style>
