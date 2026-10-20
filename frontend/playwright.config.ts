import { defineConfig, devices } from '@playwright/test';
import { existsSync } from "fs";

const chromiumPath = existsSync("/usr/bin/chromium")
  ? "/usr/bin/chromium"
  : undefined;

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: './tests',
  testMatch: '**/*.spec.ts',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  workers: process.env.CI ? 1 : undefined,
  reporter: [['html', { host: '127.0.0.1' }]],
  use: {
    baseURL: 'http://127.0.0.1:8001',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: chromiumPath ? { executablePath: chromiumPath } : {},
      },
    },
  ],

  webServer: [
    {
      command: 'cd .. && cargo run --bin test_server',
      url: 'http://127.0.0.1:3000/health',
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'pnpm dev',
      url: 'http://127.0.0.1:8001',
      reuseExistingServer: !process.env.CI,
    },
  ],
});
