import { expect, test } from '@playwright/test'
import { useLocale } from './helpers'

test.beforeEach(async ({ page }) => {
  await useLocale(page, 'en')
})

// M-02: below 16px iOS Safari auto-zooms the page when the textarea is focused.
// The fix is a font size, which we can assert directly regardless of engine.
test('the composer text is at least 16px so iOS does not auto-zoom', async ({ page }) => {
  await page.goto('/')
  const size = await page.locator('main textarea').evaluate((el) => Number.parseFloat(getComputedStyle(el).fontSize))
  expect(size).toBeGreaterThanOrEqual(16)
})

// M-01: the language switcher is in the mobile header, not only the menu.
test('the language switcher is reachable from the mobile header', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('header').getByRole('group', { name: 'Language' })).toBeVisible()
})

test('nothing overflows horizontally at 320px', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 640 })
  await page.goto('/')
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  )
  expect(overflow, 'horizontal overflow at 320px').toBeLessThanOrEqual(1)
})
