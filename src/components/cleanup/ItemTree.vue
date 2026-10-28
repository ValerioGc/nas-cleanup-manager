<script setup lang="ts">

import { computed, ref } from 'vue';
import { useI18n } from '@/i18n';
import { formatBytes } from '@valeriogc/nas-base-app';
import { GROUPS, groupName, itemName, riskyNote } from '@/services/items';
import type { CatalogEntry, ItemResult } from '@/types/cleanup';
import chevron from '@/assets/icons/chevron.svg?url';
import warning from '@/assets/icons/warning.svg?url';

/**
 * Grouped item switches. A group switch toggles all its items and shows a partial state when only
 * some of them are selected.
 */
const props = withDefaults(
  defineProps<{
    entries: CatalogEntry[];
    results?: Record<string, ItemResult>;
    disabled?: boolean;
    label: string;
  }>(),
  { results: () => ({}), disabled: false },
);

const selected = defineModel<string[]>({ required: true });
const active = defineModel<string | null>('active', { default: null });
const { locale, t } = useI18n();
const closed = ref<string[]>([]);

const groups = computed(() =>
  GROUPS.map((group) => ({
    group,
    members: props.entries.filter((entry) => entry.group === group),
  })).filter((group) => group.members.length > 0),
);

function taken(entries: CatalogEntry[]) {
  return entries.filter((entry) => selected.value.includes(entry.id)).length;
}

function partial(entries: CatalogEntry[]) {
  const count = taken(entries);
  return count > 0 && count < entries.length;
}

function toggleGroup(entries: CatalogEntry[]) {
  const ids = entries.map((entry) => entry.id);
  selected.value =
    taken(entries) === entries.length
      ? selected.value.filter((id) => !ids.includes(id))
      : [...selected.value, ...ids.filter((id) => !selected.value.includes(id))];
}

function toggleItem(id: string) {
  selected.value = selected.value.includes(id)
    ? selected.value.filter((item) => item !== id)
    : [...selected.value, id];
}

function toggleOpen(group: string) {
  closed.value = closed.value.includes(group)
    ? closed.value.filter((item) => item !== group)
    : [...closed.value, group];
}

function size(id: string) {
  const result = props.results[id];
  return result && !result.error ? formatBytes(result.bytes, locale.value) : '';
}

</script>

<template>
  <fieldset class="item_tree">
    <legend class="visually_hidden">{{ label }}</legend>
    <section v-for="{ group, members } in groups" :key="group" class="tree_group">
      <div class="group_row">
        <button
          type="button"
          class="app_button app_button--icon app_button--small group_toggle"
          :aria-expanded="!closed.includes(group)"
          :aria-label="
            (closed.includes(group) ? t.cleanup.expand : t.cleanup.collapse).replace(
              '{name}',
              groupName(t, group),
            )
          "
          @click="toggleOpen(group)"
        >
          <img
            :class="['app_icon', 'group_chevron', { group_chevron_open: !closed.includes(group) }]"
            :src="chevron"
            alt=""
            aria-hidden="true"
          />
        </button>
        <label
          :class="[
            'app_switch',
            'group_switch',
            { app_switch_disabled: disabled, group_switch_partial: partial(members) },
          ]"
          :data-testid="`group_${group}_switch`"
        >
          <input
            type="checkbox"
            :name="`group_${group}`"
            :checked="taken(members) === members.length"
            :indeterminate="partial(members)"
            :disabled="disabled"
            :data-testid="`group_${group}`"
            @change="toggleGroup(members)"
          />
          <span class="app_switch_track" aria-hidden="true"></span>
          <strong>{{ groupName(t, group) }}</strong>
        </label>
        <span class="group_count">
          {{
            t.cleanup.taken
              .replace('{taken}', String(taken(members)))
              .replace('{total}', String(members.length))
          }}
        </span>
      </div>
      
      <ul v-if="!closed.includes(group)" class="tree_items">
        <li
          v-for="entry in members"
          :key="entry.id"
          :class="['tree_item', { tree_item_active: active === entry.id }]"
        >
          <div class="item_row">
            <label
              :class="['app_switch', { app_switch_disabled: disabled }]"
              :data-testid="`item_${entry.id}_switch`"
            >
              <input
                type="checkbox"
                role="switch"
                :name="`item_${entry.id}`"
                :checked="selected.includes(entry.id)"
                :aria-checked="selected.includes(entry.id)"
                :disabled="disabled"
                :data-testid="`item_${entry.id}`"
                @change="toggleItem(entry.id)"
              />
              <span class="app_switch_track" aria-hidden="true"></span>
              <span class="visually_hidden">{{ itemName(t, entry) }}</span>
            </label>
            <button
              type="button"
              class="item_name"
              :aria-pressed="active === entry.id"
              :data-testid="`show_${entry.id}`"
              @click="active = entry.id"
            >
              {{ itemName(t, entry) }}
              <img
                v-if="entry.risky"
                class="app_icon"
                :src="warning"
                :alt="t.risky"
                :title="t.risky"
              />
            </button>
            <span class="item_size">{{ size(entry.id) }}</span>
          </div>
          <p
            v-if="entry.risky && selected.includes(entry.id)"
            class="alert_warning risky_note"
            role="note"
            :data-testid="`risky_${entry.id}`"
          >
            <img class="app_icon" :src="warning" alt="" aria-hidden="true" />
            <span>
              <strong>{{ t.riskyTitle }}</strong>
              {{ riskyNote(t, entry) }}
            </span>
          </p>
        </li>
      </ul>
    </section>
  </fieldset>
</template>

<style scoped lang="scss">

.item_tree {
  display: grid;
  gap: 10px;
  min-width: 0;
  margin: 0;
  padding: 0;
  border: 0;
}

.tree_group {
  border: 1px solid var(--nas-border);
  border-radius: 10px;
  background: var(--nas-surface);
}

.group_row,
.item_row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group_row {
  padding: 8px 10px;
}

.group_toggle {
  --app-icon-size: 16px;
  width: 32px;
  min-width: 32px;
  border-color: transparent;
}

.group_chevron {
  transition: transform 0.16s ease;
}

.group_chevron_open {
  transform: rotate(90deg);
}

.group_switch {
  flex: 1;
  min-width: 0;
}

// Partial selection puts the knob halfway.
.group_switch_partial .app_switch_track {
  background: color-mix(in srgb, var(--accent) 45%, var(--surface-border));

  &::after {
    transform: translate(6px, -50%);
  }
}

.group_count {
  color: var(--nas-muted);
  font-size: 0.85rem;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.tree_items {
  @extend %reset_list;
  display: grid;
  padding: 0 10px 8px 42px;
}

.tree_item {
  border-radius: 8px;
}

.tree_item_active {
  background: var(--nas-hover);
}

.item_row {
  padding: 4px 8px;
}

.item_name {
  --app-icon-size: 16px;
  display: inline-flex;
  flex: 1;
  align-items: center;
  gap: 6px;
  min-width: 0;
  padding: 4px 0;
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
}

.item_size {
  color: var(--nas-muted);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.risky_note {
  margin: 2px 8px 8px 52px;
}

.visually_hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}

</style>
