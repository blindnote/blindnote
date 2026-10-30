import { defineConfig, devices } from '@playwright/test'

const PORT = Number(process.env.BLINDNOTE_E2E_PORT ?? 8099)
const BASE_URL = `http://127.0.0.1:${PORT}`

/**
 * End-to-end tests run against the real server binary with the client embedded
 * in it -- the shipped artifact, not a dev server. One shared server and one
 * SQLite writer, so the suite is serial and deterministic rather than fast.
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['github'], ['list']] : [['list']],
  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'desktop', use: { ...devices['Desktop Chrome'] }, testIgnore: /mobile\.spec\.ts/ },
    {
      name: 'mobile',
      testMatch: /mobile\.spec\.ts/,
      use: { ...devices['Desktop Chrome'], viewport: { width: 375, height: 812 }, isMobile: true, hasTouch: true },
    },
  ],
  webServer: {
    command: 'bash e2e/serve.sh',
    url: `${BASE_URL}/healthz`,
    timeout: 600_000,
    reuseExistingServer: !process.env.CI,
    env: { BLINDNOTE_E2E_PORT: String(PORT) },
  },
})
