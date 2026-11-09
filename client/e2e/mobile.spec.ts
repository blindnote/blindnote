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

// M-01: the language and theme switchers live in the mobile menu, which has
// room for comfortable controls instead of cramming them into the header.
test('the language and theme switchers are reachable from the mobile menu', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Open menu' }).click()
  const menu = page.locator('#mobile-menu')
  await expect(menu.getByRole('group', { name: 'Language' })).toBeVisible()
  await expect(menu.getByRole('group', { name: 'Theme' })).toBeVisible()
})

test('nothing overflows horizontally at 320px', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 640 })
  await page.goto('/')
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  )
  expect(overflow, 'horizontal overflow at 320px').toBeLessThanOrEqual(1)
})

// The footer's language and theme switchers are the main touch surface for
// those preferences on a phone, so each segment is a comfortable tap target
// (Apple HIG 44pt) at mobile width, shrinking to a compact pill only on wide
// screens where a mouse points.
test('the footer switcher controls are a comfortable tap target', async ({ page }) => {
  await page.goto('/')
  const buttons = page.locator('footer [role="group"] button')
  const count = await buttons.count()
  expect(count, 'EN/TR plus system/light/dark').toBeGreaterThanOrEqual(5)
  for (let i = 0; i < count; i++) {
    const box = await buttons.nth(i).boundingBox()
    expect(box?.height ?? 0, `footer control ${i} tap height`).toBeGreaterThanOrEqual(44)
  }
})

// The compact create button that appears in the mobile header is a primary
// action, so it is a full 44px tap target like the rest, not the 36px it once
// was.
test('the mobile header create button is a comfortable tap target', async ({ page }) => {
  await page.goto('/#missing/missing')
  const create = page.locator('header').getByRole('button', { name: /create/i })
  await expect(create).toBeVisible()
  const box = await create.boundingBox()
  expect(box?.height ?? 0, 'header create button tap height').toBeGreaterThanOrEqual(44)
})
