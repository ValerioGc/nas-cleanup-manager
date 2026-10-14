import { expect, it } from 'vitest';
import { describeSchedule, formatBytes, scheduleKind, share } from '@valeriogc/nas-base-app';
import { groupName, itemDescription, itemName, itemTouches, riskyNote } from '@/services/items';
import { messages } from '@/i18n';
import { catalog, settings } from '@/tests/fixtures/cleanup';

it('says sizes and shares as a person reads them', () => {
  expect(formatBytes(512, 'it')).toBe('512 B');
  expect(formatBytes(1536, 'it')).toBe('1,5 KB');
  expect(formatBytes(1536, 'en')).toBe('1.5 KB');
  expect(formatBytes(5 * 1024 ** 5, 'en')).toBe('5,120.0 TB');
  expect(share(25, 100)).toBe(25);
  expect(share(200, 100)).toBe(100);
  expect(share(1, 0)).toBe(0);
});

it('names and describes the items, the caches by their own name', () => {
  const t = messages.it;
  const [archives, , , journal, temporary, cache] = catalog();
  expect(itemName(t, archives!)).toBe('Pacchetti scaricati');
  expect(itemName(t, { id: 'unknown' })).toBe('unknown');
  expect(itemName(t, cache!)).toBe('Miniature di Explorer');
  expect(itemDescription(t, temporary!, settings(), 'it')).toContain('7 giorni');
  expect(itemDescription(t, journal!, settings(), 'it')).toContain('500,0 MB');
  expect(itemDescription(t, cache!, settings(), 'it')).toBe(
    'Dati temporanei di nas-file-manager, ricreati automaticamente dall’app quando servono.',
  );
  expect(itemTouches(t, archives!)).toBe('/var/cache/apt/archives');
  expect(itemTouches(t, cache!)).toBe('nas-file-manager');
  expect(itemTouches(t, { id: 'x', group: 'caches', risky: false })).toBe('');
  expect(itemDescription(t, { id: 'x', group: 'caches', risky: false }, settings(), 'it')).toBe(
    'Dati temporanei di , ricreati automaticamente dall’app quando servono.',
  );
  const trash = catalog().at(-1)!;
  expect(itemDescription(t, trash, settings(), 'it')).toContain(
    'Elementi nel cestino di nas-file-manager',
  );
  expect(riskyNote(t, trash)).toContain('Lo svuotamento li elimina definitivamente');
  expect(riskyNote(t, { id: 'old-kernels', group: 'kernel' })).toContain('kernel di riserva');
  expect(riskyNote(t, { id: 'journal', group: 'logs' })).toBe('');
  expect(groupName(t, 'kernel')).toBe('Kernel');
  expect(groupName(t, 'other')).toBe('other');
});

it('says when a scheduled cleanup runs', () => {
  const t = messages.it;
  expect(
    describeSchedule({ frequency: 'weekly', time: '03:00', weekday: 7 }, 'it', t.scheduleSummary),
  ).toBe('Ogni domenica alle 03:00');
  expect(
    describeSchedule(
      { frequency: 'yearly', time: '04:00', day: 2, month: 3 },
      'it',
      t.scheduleSummary,
    ),
  ).toBe('Ogni anno il 2 marzo alle 04:00');
  expect(describeSchedule({ frequency: 'daily', time: '01:00' }, 'it', t.scheduleSummary)).toBe(
    'Ogni giorno alle 01:00',
  );
  expect(scheduleKind({ frequency: 'monthly', time: '05:00' }, t.frequencies)).toBe(
    'Ogni mese · 05:00',
  );
});
