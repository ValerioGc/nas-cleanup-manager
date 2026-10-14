import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const ROOT = resolve(__dirname, '../../../..');

describe('the CSS of the build', () => {
  it('keeps the colours of the system disk, whose classes are named at runtime', () => {
    const config = readFileSync(resolve(ROOT, 'vite.config.common.ts'), 'utf8');
    const overview = readFileSync(
      resolve(ROOT, 'src/components/sections/OverviewPage.vue'),
      'utf8',
    );

    expect(overview).toContain('`disk_part_${part.id}`');
    expect(config).toContain('/^disk_part_/');
  });
});
