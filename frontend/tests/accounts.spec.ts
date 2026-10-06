import { test, expect } from '@playwright/test';

test('admin can register, login and invite users', async ({ page }) => {
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

  // Invite a user
  await page.getByRole('button', { name: 'person testadmin' }).click();
  await page.getByRole('button', { name: 'person_add Invite user' }).click();
  const dialog = page.locator('dialog[popover]');
  await expect(dialog).toBeVisible();
  const secondToken = (await dialog.locator('pre').textContent())?.trim();
  expect(typeof secondToken === 'string', 'some comment').toBe(true);

  // Logout
  await page.getByRole('button', { name: 'person testadmin' }).click();
  await page.getByRole('button', { name: 'logout Log out' }).click();
  await expect(page.locator(".artwork-view")).toBeVisible();

  // stays logged out
  await page.reload();
  await expect(page.locator(".artwork-view")).toBeVisible();

  // register user
  await page.getByRole('link', { name: 'person_add Register' }).click();
  await page.getByPlaceholder(' ').first().click();
  await page.getByPlaceholder(' ').first().fill(secondToken!);
  await page.getByPlaceholder(' ').nth(1).click();
  await page.getByPlaceholder(' ').nth(1).fill('testuser');
  await page.getByPlaceholder(' ').nth(1).press('Tab');
  await page.locator('input[type="email"]').fill(`testuser+${secondToken}@user.de`);
  await page.locator('input[type="email"]').press('Tab');
  await page.locator('input[type="password"]').fill('123&testüser"#!awd90');
  await page.locator('input[type="password"]').press('Tab');
  await page.getByRole('button', { name: 'add Create Account' }).press('Enter');
  await expect(page.getByRole('button', { name: 'person testuser' })).toBeVisible();

  // user logout
  await page.getByRole('button', { name: 'person testuser' }).click();
  await page.getByRole('button', { name: 'logout Log out' }).click();
  await expect(page.locator(".artwork-view")).toBeVisible();

  // admin login (!= registration)
  await page.getByRole('link', { name: 'lock_open Sign In' }).click();
  await page.locator('input[type="email"]').click();
  await page.locator('input[type="email"]').fill(`testadmin+${initialToken}@admin.de`);
  await page.locator('input[type="email"]').press('Tab');
  await page.locator('input[type="password"]').fill('adminadmin');
  await page.getByRole('button', { name: 'arrow_forward Sign In' }).click();
  await expect(page.getByRole('button', { name: 'person testadmin' })).toBeVisible();
});
