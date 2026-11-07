import { expect, test } from '@playwright/test';

test('the overview shows the system disk, the data disks and the way to the cleanup', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Panoramica');
  await expect(page.getByTestId('system_disk')).toContainText('20,0 GB liberi su 64,0 GB');
  await expect(page.getByTestId('category_caches')).toContainText('64,0 MB');
  await expect(page.getByText('Dati', { exact: true })).toBeVisible();
  await page.getByTestId('go_cleanup').click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Pulizia');
});

test('a cleanup is previewed, confirmed and kept in the history', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('section_cleanup').click();
  await expect(page.getByTestId('item_old-kernels')).not.toBeChecked();
  await expect(page.getByTestId('item_temporary')).toBeChecked();
  await page.getByTestId('item_old-kernels_switch').click();
  await expect(page.getByTestId('item_old-kernels')).toBeChecked();
  await expect(page.getByTestId('risky_old-kernels')).toContainText('kernel di riserva');
  await page.getByTestId('cleanup_preview').click();
  await expect(page.getByTestId('cleanup_total')).toContainText('Totale liberabile');
  await page.getByTestId('show_old-kernels').click();
  await expect(page.getByTestId('item_detail')).toContainText('linux-image-6.1.0-9-amd64');
  await page.getByTestId('cleanup_start').click();
  await expect(page.getByRole('dialog')).toContainText('Kernel vecchi');
  await page.getByTestId('cleanup_confirm').click();
  await expect(page.getByTestId('cleanup_outcome')).toContainText('Pulizia completata');

  await page.reload();
  await page.getByTestId('section_cleanup').click();
  await expect(page.getByTestId('item_old-kernels')).toBeChecked();

  await page.getByTestId('section_history').click();
  await expect(page.getByText('A mano').first()).toBeVisible();
  await expect(page.getByText('Kernel vecchi').first()).toBeVisible();
});

test('a scheduled cleanup is created, turned off and deleted', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('section_schedule').click();
  await expect(page.getByTestId('rules_empty')).toBeVisible();
  await page.getByTestId('rule_add').click();
  await page.getByTestId('rule_name').fill('Ogni notte');
  await page.getByTestId('rule_frequency').selectOption('daily');
  await page.getByTestId('rule_time').fill('02:30');
  await page.getByTestId('rule_save').click();
  const row = page.locator('tr', { hasText: 'Ogni notte' });
  await expect(row).toContainText('Ogni giorno alle 02:30');
  await row.getByTestId(/^rule_switch_.+_switch$/).click();
  await expect(row.getByRole('switch')).not.toBeChecked();
  await expect(row).toContainText('Disattivata');
  await row.getByRole('button', { name: 'Elimina Ogni notte' }).click();
  await page.getByTestId('rule_delete_confirm').click();
  await expect(page.getByTestId('rules_empty')).toBeVisible();
});

test('the settings change the numbers of the cleanup', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('open_settings').click();
  await page.getByTestId('settings_days').fill('14');
  await page.getByTestId('settings_save').click();
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await page.getByTestId('section_cleanup').click();
  await page.getByTestId('show_temporary').click();
  await expect(page.getByTestId('item_detail')).toContainText('14 giorni');
});

test('turning on an item at the end of the list scrolls only the list', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 560 });
  await page.goto('/');
  await page.getByTestId('section_cleanup').click();
  const tree = page.getByRole('group', { name: 'Voci da pulire' });
  await tree.evaluate((element) => element.scrollTo(0, element.scrollHeight));
  expect(await tree.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);

  for (const [role, name] of [
    ['switch', 'Cestino di Explorer'],
    ['checkbox', 'Cestini'],
    ['switch', 'Miniature di Explorer'],
    ['checkbox', 'Cache delle app'],
  ] as const) {
    const control = page.getByRole(role, { name, exact: true });
    const before = await control.isChecked();
    await page.locator('label', { has: control }).click();
    await expect(control).toBeChecked({ checked: !before });
    const scrolled = await tree.evaluate((list) =>
      [document.scrollingElement, ...document.querySelectorAll('*')]
        .filter((element) => element && element !== list && element.scrollTop > 0)
        .map((element) => element?.className),
    );
    expect(scrolled).toEqual([]);
    await expect(page.getByRole('heading', { level: 1 })).toBeInViewport();
    await expect(page.getByTestId('cleanup_start')).toBeInViewport();
  }
});
