import { test, expect } from '@playwright/test';

test('Can share projects publicly', async ({ page }) => {
  const initialToken = await (await fetch('http://127.0.0.1:3000/testGetInitialRegistrationToken')).json();
  await page.goto('/');
  await expect(page.locator(".artwork-view")).toBeVisible();

  // Register account
  await page.getByRole('link', { name: 'person_add Register' }).click();
  await page.getByPlaceholder(' ').first().click();
  await page.getByPlaceholder(' ').first().fill(initialToken);
  await page.getByPlaceholder(' ').nth(1).click();
  await page.getByPlaceholder(' ').nth(1).fill('testadmin');
  await page.getByPlaceholder(' ').nth(1).press('Tab');
  await page.locator('input[type="email"]').fill(`testadmin+${initialToken}@admin.de`);
  await page.locator('input[type="email"]').press('Tab');
  await page.locator('input[type="password"]').fill('adminadmin');
  await page.locator('input[type="password"]').press('Tab');
  await page.getByRole('button', { name: 'add Create Account' }).press('Enter');
  await expect(page.getByRole('button', { name: 'person testadmin' })).toBeVisible();

  // Enable open shares
  await page.getByRole('button', { name: 'person testadmin' }).click();
  await page.getByRole('button', { name: 'settings Settings' }).click();
  await page.getByRole('button', { name: 'admin_panel_settings' }).click();
  let settings = await page.evaluate(async () => await (await fetch('/api/v1/settings')).json());
  if (!settings.openSharing) {
    await page.getByText('Allow External Sharing').click();
  }
  settings = await page.evaluate(async () => await (await fetch('/api/v1/settings')).json());
  expect(settings.openSharing).toBeTruthy();
  await page.getByRole('button', { name: 'home' }).click();

  // Create project
  await page.getByRole('button', { name: 'add Create' }).first().click();
  await page.getByRole('textbox', { name: 'Project title...' }).fill('Sample project');
  await page.getByRole('button', { name: 'done Create' }).click();
  await expect(page.getByRole('link', { name: 'description Sample project' })).toBeVisible();

  // Share project
  await page.getByRole('button', { name: 'more_vert' }).first().click();
  await page.getByRole('button', { name: 'settings Project Settings' }).click();
  await page.getByRole('button', { name: 'share Invites & Members' }).click();
  await page.getByPlaceholder(' ').nth(1).click();
  const shareId = `test-share-id-${Math.random()}-${new Date().getMilliseconds()}`;
  await page.getByPlaceholder(' ').nth(1).fill(shareId);
  await page.locator('input[type="password"]').click();
  await page.locator('input[type="password"]').press('ControlOrMeta+a');
  await page.locator('input[type="password"]').fill('test-share-password');
  await page.locator('input[type="password"]').click({
    modifiers: ['ControlOrMeta']
  });
  await page.getByRole('button', { name: 'arrow_forward Share Now' }).click();
  await expect(page.getByRole('button', { name: 'arrow_forward Unshare & Kick' })).toBeVisible();
  await expect(page.getByText(`http://127.0.0.1:8001/joinSingleProject/${shareId}`)).toBeVisible();
  await page.getByRole('button', { name: 'close' }).nth(1).click();

  // Logout
  await page.getByRole('button', { name: 'person testadmin' }).click();
  await page.getByRole('button', { name: 'logout Log out' }).click();
  await expect(page.locator(".artwork-view")).toBeVisible();

  // register user
  await page.getByRole('link', { name: 'heart_smile Join Project' }).click();
  await page.getByPlaceholder(' ').first().click();
  await page.getByPlaceholder(' ').first().fill(shareId);
  await page.locator('input[type="password"]').click();
  await page.locator('input[type="password"]').fill('test-share-password');
  await page.getByRole('button', { name: 'arrow_forward Open Project' }).click();

  // Create File
  await page.getByRole('button', { name: 'note_add' }).click();
  await page.getByRole('textbox', { name: 'main.tex' }).fill('test.typ');
  await page.getByRole('textbox', { name: 'main.tex' }).press('Enter');
  await page.getByText('descriptiontest.typmore_vert').click();

  // Compiles
  await page.locator('a').filter({ hasText: 'Compile' }).click();
  await page.getByRole('button', { name: 'check' }).click();
  await expect(page.getByText('Exited with code:')).toBeVisible();

  // Home screen has disabled button
  await page.getByRole('button', { name: 'home' }).click();
  await expect(page.getByRole('button', { name: 'add Create' })).toBeDisabled();

  // user logout
  await page.getByRole('button').first().click();
  await page.getByRole('button', { name: 'logout Log out' }).click();
  await expect(page.locator(".artwork-view")).toBeVisible();

  // admin sees changes
  await page.getByRole('link', { name: 'lock_open Sign In' }).click();
  await page.locator('input[type="email"]').click();
  await page.locator('input[type="email"]').fill(`testadmin+${initialToken}@admin.de`);
  await page.locator('input[type="email"]').press('Tab');
  await page.locator('input[type="password"]').fill('adminadmin');
  await page.getByRole('button', { name: 'arrow_forward Sign In' }).click();
  await page.getByRole('link', { name: 'description Sample project' }).click();
  await expect(page.getByText('test.typ')).toBeVisible();
});
