import { defineConfig } from 'vitest/config'

// Pure-logic unit tests (crypto, link, base64url). No DOM or React is needed,
// so the node environment is enough -- WebCrypto, btoa/atob and TextEncoder are
// all Node globals. Browser-level flows are covered by the Playwright E2E suite.
export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
})
