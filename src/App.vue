<script setup lang="ts">

import { computed, ref } from 'vue';
import { NasAppLayout, type NasCategory } from '@valeriogc/nas-base-app';
import { useNasLauncherContext } from '@/composables/useNasLauncherContext';
import type { SectionCode } from '@/types/cleanup';
import SettingsModal from '@/components/SettingsModal.vue';
import CleanupPage from '@/components/sections/CleanupPage.vue';
import HistoryPage from '@/components/sections/HistoryPage.vue';
import OverviewPage from '@/components/sections/OverviewPage.vue';
import SchedulePage from '@/components/sections/SchedulePage.vue';
import cleanup from '@/assets/icons/cleanup.svg?url';
import history from '@/assets/icons/history.svg?url';
import overview from '@/assets/icons/overview.svg?url';
import scheduled from '@/assets/icons/scheduled.svg?url';
import settings from '@/assets/icons/settings.svg?url';

const { t } = useNasLauncherContext();
const section = ref<SectionCode>('overview');
const settingsOpen = ref(false);
const icons: Record<SectionCode, string> = { overview, cleanup, schedule: scheduled, history };
const sections = computed<NasCategory<SectionCode>[]>(() =>
  (['overview', 'cleanup', 'schedule', 'history'] as const).map((code) => ({
    value: code,
    label: t.value.sections[code],
    icon: icons[code],
    testid: `section_${code}`,
  })),
);

</script>

<template>
  <!-- The cleanup scrolls only its list: the details and the buttons stay in view. -->
  <NasAppLayout
    v-model="section"
    :categories="sections"
    :navigation-label="t.sectionsLabel"
    :contained="section === 'cleanup'"
  >
    <template #icon="{ category }">
      <img class="app_icon category_icon" :src="category.icon" alt="" aria-hidden="true" />
    </template>
    <template #sidebar-footer>
      <button
        type="button"
        class="category_button settings_item"
        data-testid="open_settings"
        @click="settingsOpen = true"
      >
        <img class="app_icon category_icon" :src="settings" alt="" aria-hidden="true" />
        {{ t.settings }}
      </button>
    </template>
    <OverviewPage v-if="section === 'overview'" @cleanup="section = 'cleanup'" />
    <CleanupPage v-else-if="section === 'cleanup'" />
    <SchedulePage v-else-if="section === 'schedule'" />
    <HistoryPage v-else />
    <!-- Inside the layout, where the shared styles of the controls and the toolbars apply. -->
    <SettingsModal v-if="settingsOpen" v-model="settingsOpen" />
  </NasAppLayout>
</template>

<style scoped lang="scss">

// Down at the bottom, apart from the sections.
.settings_item {
  margin-top: auto;
}

</style>
