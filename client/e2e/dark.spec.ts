import { expect, test } from '@playwright/test'

// Dark mode follows the OS colour scheme with no JavaScript (styles/app.css
// wires the `dark` variant to prefers-color-scheme). These guard that wiring
// end-to-end against the shipped binary: the gate was once an explicit
// data-theme and it would be easy to re-break.

test('the page paints dark under a dark colour scheme', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.goto('/')
  const background = await page.evaluate(() => getComputedStyle(document.body).backgroundColor)
  expect(background).toBe('rgb(26, 18, 13)') // --color-night #1a120d
})

test('the page paints light under a light colour scheme', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' })
  await page.goto('/')
  const background = await page.evaluate(() => getComputedStyle(document.body).backgroundColor)
  expect(background).toBe('rgb(255, 255, 255)') // --color-page #ffffff
})

test('the browser chrome colour is declared for both schemes', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('meta[name="theme-color"][media*="light"]')).toHaveAttribute('content', '#fef8f2')
  await expect(page.locator('meta[name="theme-color"][media*="dark"]')).toHaveAttribute('content', '#1a120d')
})
