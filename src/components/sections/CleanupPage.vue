<script setup lang="ts">

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useI18n } from '@/i18n';
import {
  fetchCatalog,
  fetchHistory,
  fetchProgress,
  fetchSelection,
  fetchSettings,
  previewItems,
  saveSelection,
  startCleanup,
} from '@/services/api';
import { formatBytes, NasLoading } from '@valeriogc/nas-base-app';
import { itemDescription, itemName, itemTouches, riskyNote } from '@/services/items';
import type { CatalogEntry, ItemResult, Progress, Settings } from '@/types/cleanup';

import CommonModal from '@/components/common/CommonModal.vue';
import ItemTree from '@/components/cleanup/ItemTree.vue';

import cleanupIcon from '@/assets/icons/cleanup.svg?url';
import previewIcon from '@/assets/icons/preview.svg?url';
import warning from '@/assets/icons/warning.svg?url';

// Progress polling interval while a cleanup runs.
const PROGRESS_MS = 1000;

const { locale, t } = useI18n();
const loading = ref(true);
const error = ref<string | null>(null);
const entries = ref<CatalogEntry[]>([]);
const selected = ref<string[]>([]);
const active = ref<string | null>(null);
const settings = ref<Settings>({ temporaryDays: 7, journalMb: 500, notify: true });
const results = ref<Record<string, ItemResult>>({});
const previewing = ref(false);
const confirmOpen = ref(false);
const progress = ref<Progress | null>(null);
const outcome = ref<string | null>(null);

let timer: ReturnType<typeof setTimeout> | undefined;
let loaded = false;

const shown = computed(() => entries.value.find((entry) => entry.id === active.value) ?? null);
const shownResult = computed(() => (active.value ? results.value[active.value] : undefined));
const running = computed(() => progress.value?.running ?? false);
const riskyChosen = computed(() =>
  entries.value.filter((entry) => entry.risky && selected.value.includes(entry.id)),
);

const total = computed(() =>
  selected.value.reduce((sum, id) => sum + (results.value[id]?.bytes ?? 0), 0),
);

const measured = computed(() => selected.value.some((id) => results.value[id] !== undefined));

function reason(failure: unknown) {
  return failure instanceof Error ? failure.message : t.value.loadFailed;
}

async function load() {
  loading.value = true;
  error.value = null;
  try {
    const [catalog, chosen, saved, current] = await Promise.all([
      fetchCatalog(),
      fetchSelection(),
      fetchSettings(),
      fetchProgress(),
    ]);
    entries.value = catalog;
    selected.value = chosen.filter((id) => catalog.some((entry) => entry.id === id));
    settings.value = saved;
    active.value = catalog[0]?.id ?? null;
    loaded = true;
    follow(current);
  } catch (error_) {
    error.value = reason(error_);
  } finally {
    loading.value = false;
  }
}

// Persist the selection for the next visit.
watch(selected, (items) => {
  if (loaded) {
    saveSelection(items).catch((error_: unknown) => (error.value = reason(error_)));
  }
});

async function preview() {
  if (!selected.value.length) {
    return;
  }

  previewing.value = true;
  error.value = null;
  outcome.value = null;
  try {
    const measuredItems = await previewItems(selected.value);
    results.value = Object.fromEntries(measuredItems.map((result) => [result.id, result]));
  } catch (error_) {
    error.value = reason(error_);
  } finally {
    previewing.value = false;
  }
}

/** Polls the progress until the cleanup ends, then reports the space freed. */
function follow(current: Progress) {
  const wasRunning = running.value;
  progress.value = current;
  clearTimeout(timer);
  if (current.running) {
    timer = setTimeout(() => {
      fetchProgress()
        .then(follow)
        .catch((error_: unknown) => (error.value = reason(error_)));
    }, PROGRESS_MS);
  } else if (wasRunning) {
    void finished();
  }
}

async function finished() {
  results.value = {};
  const [last] = await fetchHistory();
  if (!last) {
    return;
  }

  const failed = last.results.filter((result) => result.error).length;
  const size = formatBytes(last.freed, locale.value);
  outcome.value = failed
    ? t.value.cleanup.doneWithErrors.replace('{size}', size).replace('{count}', String(failed))
    : t.value.cleanup.done.replace('{size}', size);
}

async function clean() {
  confirmOpen.value = false;
  error.value = null;
  outcome.value = null;
  try {
    await startCleanup(selected.value);
    progress.value = {
      running: true,
      rule: null,
      item: null,
      done: 0,
      total: selected.value.length,
    };
    follow(await fetchProgress());
  } catch (error_) {
    error.value = reason(error_);
  }
}

onMounted(load);
onBeforeUnmount(() => clearTimeout(timer));

</script>

<template>
  <section class="cleanup_page" :aria-label="t.sections.cleanup">
    <header class="page_header">
      <h1>{{ t.sections.cleanup }}</h1>
      <p>{{ t.cleanup.intro }}</p>
    </header>
    <NasLoading v-if="loading" :label="t.loading" />
    <p v-if="error" class="app_error page_error" role="alert">{{ error }}</p>
    <output v-if="outcome" class="page_outcome" data-testid="cleanup_outcome">
      {{ outcome }}
    </output>
    <output v-if="running && progress" class="page_progress" data-testid="cleanup_progress">
      <span>
        {{
          t.cleanup.running
            .replace('{done}', String(progress.done))
            .replace('{total}', String(progress.total))
        }}
      </span>
      <progress :max="progress.total" :value="progress.done"></progress>
    </output>
    <div v-if="!loading && entries.length" class="cleanup_layout">
      <ItemTree
        v-model="selected"
        v-model:active="active"
        :entries="entries"
        :results="results"
        :disabled="running"
        :label="t.cleanup.tree"
      />
      <aside v-if="shown" class="item_detail app_panel" data-testid="item_detail">
        <h2>
          {{ itemName(t, shown) }}
          <img v-if="shown.risky" class="app_icon" :src="warning" :alt="t.risky" :title="t.risky" />
        </h2>
        <p>{{ itemDescription(t, shown, settings, locale) }}</p>
        <p v-if="shown.risky" class="alert_warning detail_risky" role="note">
          <img class="app_icon" :src="warning" alt="" aria-hidden="true" />
          <span>
            <strong>{{ t.riskyTitle }}</strong>
            {{ riskyNote(t, shown) }}
          </span>
        </p>
        <dl>
          <dt>{{ t.cleanup.touches }}</dt>
          <dd>
            <code>{{ itemTouches(t, shown) }}</code>
          </dd>
          <dt>{{ t.cleanup.frees }}</dt>
          <dd v-if="!shownResult">{{ t.cleanup.notMeasured }}</dd>
          <dd v-else-if="shownResult.error" class="app_error">
            {{ t.cleanup.failed.replace('{reason}', shownResult.error) }}
          </dd>
          <dd v-else data-testid="detail_size">{{ formatBytes(shownResult.bytes, locale) }}</dd>
        </dl>
        <template v-if="shownResult?.entries.length">
          <h3>{{ t.cleanup.entries }}</h3>
          <ul class="detail_entries">
            <li v-for="entry in shownResult.entries" :key="entry">
              <code>{{ entry }}</code>
            </li>
          </ul>
          <p v-if="shownResult.more" class="muted">
            {{ t.cleanup.more.replace('{count}', String(shownResult.more)) }}
          </p>
        </template>
      </aside>
    </div>
    <footer v-if="!loading && entries.length" class="page_actions app_toolbar">
      <span v-if="measured" class="page_total" data-testid="cleanup_total">
        {{ t.cleanup.total.replace('{size}', formatBytes(total, locale)) }}
      </span>
      <span v-else-if="!selected.length" class="muted">{{ t.cleanup.chooseOne }}</span>
      <button
        type="button"
        class="app_button"
        :disabled="!selected.length || previewing || running"
        data-testid="cleanup_preview"
        @click="preview"
      >
        <img class="app_icon" :src="previewIcon" alt="" aria-hidden="true" />
        {{ previewing ? t.cleanup.previewing : t.cleanup.preview }}
      </button>
      <button
        type="button"
        class="app_button app_button--primary"
        :disabled="!selected.length || previewing || running"
        data-testid="cleanup_start"
        @click="confirmOpen = true"
      >
        <img class="app_icon" :src="cleanupIcon" alt="" aria-hidden="true" />
        {{ t.cleanup.clean }}
      </button>
    </footer>
    <CommonModal v-model="confirmOpen" :title="t.cleanup.confirmTitle">
      <p class="confirm_message">
        {{ t.cleanup.confirmMessage.replace('{count}', String(selected.length)) }}
      </p>
      <div v-if="riskyChosen.length" class="alert_warning" role="note">
        <img class="app_icon" :src="warning" alt="" aria-hidden="true" />
        <div>
          <strong>{{ t.cleanup.confirmRisky }}</strong>
          <ul>
            <li v-for="entry in riskyChosen" :key="entry.id">{{ itemName(t, entry) }}</li>
          </ul>
        </div>
      </div>
      <p v-if="measured" class="confirm_total">
        {{ t.cleanup.total.replace('{size}', formatBytes(total, locale)) }}
      </p>
      <template #footer>
        <button type="button" class="app_button" @click="confirmOpen = false">
          {{ t.cancel }}
        </button>
        <button
          type="button"
          class="app_button app_button--primary"
          data-testid="cleanup_confirm"
          @click="clean"
        >
          <img class="app_icon" :src="cleanupIcon" alt="" aria-hidden="true" />
          {{ t.cleanup.clean }}
        </button>
      </template>
    </CommonModal>
  </section>
</template>

<style scoped lang="scss">

.cleanup_page {
  display: flex;
  flex-direction: column;
  gap: 18px;
  height: 100%;
}

.cleanup_layout {
  display: grid;
  flex: 1;
  grid-template-columns: minmax(0, 1.2fr) minmax(260px, 1fr);
  grid-template-rows: minmax(0, 1fr);
  min-height: 0;
  gap: 18px;

  > .item_tree {
    min-height: 0;
    padding-right: 4px;
    overflow-y: auto;
    align-content: start;
  }
}

.item_detail {
  --app-icon-size: 18px;
  align-self: start;
  max-height: 100%;
  padding: 18px 20px;
  overflow-y: auto;

  h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 0 8px;
    font-size: 1.05rem;
  }

  h3 {
    margin: 14px 0 6px;
    font-size: 0.95rem;
  }

  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 6px 12px;
    margin: 0;
  }

  dt {
    color: var(--nas-muted);
  }

  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
}

.detail_risky {
  margin-bottom: 12px;
}

.confirm_message {
  margin: 0 0 12px;
}

.confirm_total {
  margin: 12px 0 0;
  font-weight: 600;
}

.detail_entries {
  @extend %reset_list;
  display: grid;
  gap: 2px;
  max-height: 220px;
  overflow: auto;
  font-size: 0.85rem;
}

.page_progress {
  display: grid;
  gap: 6px;

  progress {
    width: 100%;
    accent-color: var(--accent, #3c8fbe);
  }
}

.page_outcome {
  margin: 0;
  padding: 10px 14px;
  border-left: 3px solid var(--accent, #3c8fbe);
  border-radius: 8px;
  background: var(--nas-surface);
}

.page_error {
  margin: 0;
}

.page_actions {
  justify-content: flex-end;
}

.page_total {
  margin-right: auto;
  font-weight: 600;
}

@media (max-width: 900px) {
  .cleanup_page {
    height: auto;
  }

  .cleanup_layout {
    flex: none;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: none;

    > .item_tree {
      overflow-y: visible;
    }
  }

  .item_detail {
    max-height: none;
  }
}

</style>
