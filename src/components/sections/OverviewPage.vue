<script setup lang="ts">

import { computed, onMounted, ref } from 'vue';
import { useI18n } from '@/i18n';
import { fetchOverview } from '@/services/api';
import { formatBytes, NasLoading, share } from '@valeriogc/nas-base-app';
import type { Overview } from '@/types/cleanup';

const emit = defineEmits<{ cleanup: [] }>();
const { locale, t } = useI18n();
const overview = ref<Overview | null>(null);
const error = ref<string | null>(null);
const loading = ref(true);

const CATEGORIES = ['system', 'apps', 'caches', 'logs', 'packages', 'temporary', 'free'] as const;
type Category = (typeof CATEGORIES)[number];

/** The disk of the system split by what takes it: the system is what the others leave. */
const parts = computed(() => {
  const value = overview.value;
  if (!value) {
    return [];
  }
  const counted = new Map(value.categories.map((category) => [category.id, category.bytes]));
  const others = [...counted.values()].reduce((sum, bytes) => sum + bytes, 0);
  const used = Math.max(0, value.total - value.free);
  const bytes = (category: Category) => {
    if (category === 'free') {
      return value.free;
    }
    if (category === 'system') {
      return Math.max(0, used - others);
    }
    return counted.get(category) ?? 0;
  };
  return CATEGORIES.map((id) => ({ id, bytes: bytes(id), percent: share(bytes(id), value.total) }));
});

const last = computed(() => {
  const cleanup = overview.value?.last;
  if (!cleanup) {
    return t.value.overview.never;
  }
  const date = new Intl.DateTimeFormat(locale.value, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(cleanup.at);
  return t.value.overview.lastText
    .replace('{date}', date)
    .replace('{size}', formatBytes(cleanup.freed, locale.value));
});

async function load() {
  loading.value = true;
  error.value = null;
  try {
    overview.value = await fetchOverview();
  } catch (failure) {
    error.value = failure instanceof Error ? failure.message : t.value.loadFailed;
  } finally {
    loading.value = false;
  }
}

onMounted(load);

</script>

<template>
  <section class="overview_page" :aria-label="t.sections.overview">
    <header class="page_header">
      <h1>{{ t.sections.overview }}</h1>
    </header>
    <NasLoading v-if="loading" :label="t.loading" />
    <div v-else-if="error" class="app_toolbar" role="alert">
      <span class="app_error">{{ error }}</span>
      <button type="button" class="app_button app_button--small" @click="load">
        {{ t.retry }}
      </button>
    </div>
    <template v-if="overview">
      <article class="app_panel overview_card" data-testid="system_disk">
        <h2>{{ t.overview.systemDisk }}</h2>
        <p class="muted">
          {{
            t.overview.space
              .replace('{free}', formatBytes(overview.free, locale))
              .replace('{total}', formatBytes(overview.total, locale))
          }}
        </p>
        <div class="disk_bar" role="img" :aria-label="t.overview.systemDisk">
          <span
            v-for="part in parts"
            :key="part.id"
            :class="['disk_part', `disk_part_${part.id}`]"
            :style="{ width: `${part.percent}%` }"
            :title="`${t.overview.categories[part.id]} ${formatBytes(part.bytes, locale)}`"
          ></span>
        </div>
        <ul class="legend">
          <li v-for="part in parts" :key="part.id" :data-testid="`category_${part.id}`">
            <span :class="['legend_dot', `disk_part_${part.id}`]" aria-hidden="true"></span>
            {{ t.overview.categories[part.id] }}
            <strong>{{ formatBytes(part.bytes, locale) }}</strong>
          </li>
        </ul>
      </article>
      <article class="app_panel overview_card">
        <h2>{{ t.overview.dataDisks }}</h2>
        <p class="muted">{{ t.overview.dataDisksNote }}</p>
        <p v-if="!overview.disks.length">{{ t.overview.noDisks }}</p>
        <ul v-else class="data_disks">
          <li v-for="disk in overview.disks" :key="disk.name">
            <span>{{ disk.name }}</span>
            <meter :min="0" :max="disk.total" :value="disk.total - disk.free"></meter>
            <span class="muted">
              {{
                t.overview.space
                  .replace('{free}', formatBytes(disk.free, locale))
                  .replace('{total}', formatBytes(disk.total, locale))
              }}
            </span>
          </li>
        </ul>
      </article>
      <article class="app_panel overview_card last_card">
        <div>
          <h2>{{ t.overview.last }}</h2>
          <p data-testid="last_cleanup">{{ last }}</p>
        </div>
        <button
          type="button"
          class="app_button app_button--primary"
          data-testid="go_cleanup"
          @click="emit('cleanup')"
        >
          {{ t.overview.toCleanup }}
        </button>
      </article>
    </template>
  </section>
</template>

<style scoped lang="scss">

.overview_page {
  display: grid;
  gap: 18px;
}

.overview_card {
  padding: 18px 20px;

  h2 {
    margin: 0 0 4px;
    font-size: 1.05rem;
  }
}

.disk_bar {
  display: flex;
  height: 18px;
  margin: 12px 0;
  overflow: hidden;
  border-radius: 9px;
  background: var(--nas-sunken);
}

.disk_part {
  height: 100%;
}

.disk_part_system {
  background: #5b7a99;
}

.disk_part_apps {
  background: #3c8fbe;
}

.disk_part_caches {
  background: #8e6cc0;
}

.disk_part_logs {
  background: #c9752e;
}

.disk_part_packages {
  background: #d4a72c;
}

.disk_part_temporary {
  background: #c25b78;
}

.disk_part_free {
  background: var(--nas-sunken);
}

.legend {
  @extend %reset_list;
  display: flex;
  flex-wrap: wrap;
  gap: 8px 18px;

  li {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
}

.legend_dot {
  display: inline-block;
  flex: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

// The free room is the empty track of the bar: its dot needs an edge to show.
.legend_dot.disk_part_free {
  border: 1px solid var(--nas-border);
}

.data_disks {
  @extend %reset_list;
  display: grid;
  gap: 8px;

  li {
    display: grid;
    grid-template-columns: minmax(80px, 160px) minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
  }

  meter {
    width: 100%;
  }
}

.last_card {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;

  p {
    margin: 0;
  }
}

</style>
