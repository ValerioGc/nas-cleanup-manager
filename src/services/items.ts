import type { messages } from '@/i18n';
import type { CatalogEntry, Settings } from '@/types/cleanup';
import { formatBytes } from '@valeriogc/nas-base-app';

type Catalog = (typeof messages)['it'];

/** The groups of the tree, in the order they are shown. */
export const GROUPS = ['apt', 'kernel', 'logs', 'temporary', 'nas', 'caches', 'trash'] as const;

function system(t: Catalog, id: string) {
  return (t.items as Record<string, { name: string; description: string; touches: string }>)[id];
}

/** The name of an item: the one of the catalogue, or the one a cache gives itself. */
export function itemName(t: Catalog, entry: Pick<CatalogEntry, 'id' | 'label'>): string {
  return entry.label ?? system(t, entry.id)?.name ?? entry.id;
}

/** What an item does, with the numbers of the settings in it. */
export function itemDescription(
  t: Catalog,
  entry: CatalogEntry,
  settings: Settings,
  locale: string,
): string {
  const known = system(t, entry.id);
  if (!known) {
    const text = entry.group === 'trash' ? t.trashDescription : t.cacheDescription;
    return text.replace('{app}', entry.app ?? '');
  }
  return known.description
    .replace('{days}', String(settings.temporaryDays))
    .replace('{size}', formatBytes(settings.journalMb * 1024 * 1024, locale));
}

export function itemTouches(t: Catalog, entry: CatalogEntry): string {
  return system(t, entry.id)?.touches ?? entry.app ?? '';
}

/** Why a delicate item is delicate: a trash says it for every one. */
export function riskyNote(t: Catalog, entry: Pick<CatalogEntry, 'id' | 'group'>): string {
  return t.riskyNotes[entry.id] ?? (entry.group === 'trash' ? t.trashRiskyNote : '');
}

export function groupName(t: Catalog, group: string): string {
  return (t.groups as Record<string, string>)[group] ?? group;
}
