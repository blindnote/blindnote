import { expect, type Page, test } from '@playwright/test'
import { useLocale } from './helpers'

// The footer toggle lets a visitor override the OS colour scheme. A light/dark
// choice is applied before first paint by an inline bootstrap in index.html,
// pinned in the CSP by the hash of its text (build.rs). So these also prove the
// strict CSP actually allows that one inline script: were it blocked, the
// choice would not survive a reload and the browser would log a CSP violation.

/** The body's painted background, as the browser resolves it. */
function bodyBackground(page: Page): Promise<string> {
  return page.evaluate(() => getComputedStyle(document.body).backgroundColor)
}

/** A button inside the footer's theme toggle, by its label. */
function themeButton(page: Page, name: 'System' | 'Light' | 'Dark') {
  return page.getByRole('group', { name: 'Theme' }).getByRole('button', { name })
}

const NIGHT = 'rgb(26, 18, 13)' // --color-night #1a120d
const PAGE = 'rgb(255, 255, 255)' // --color-page #ffffff

test('a light override paints light even under a dark OS scheme', async ({ page }) => {
  await useLocale(page, 'en')
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.goto('/')
  expect(await bodyBackground(page)).toBe(NIGHT)

  await themeButton(page, 'Light').click()
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')
  expect(await bodyBackground(page)).toBe(PAGE)
})

test('a dark override paints dark even under a light OS scheme', async ({ page }) => {
  await useLocale(page, 'en')
  await page.emulateMedia({ colorScheme: 'light' })
  await page.goto('/')
  expect(await bodyBackground(page)).toBe(PAGE)

  await themeButton(page, 'Dark').click()
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
  expect(await bodyBackground(page)).toBe(NIGHT)
})

test('system removes the override and follows the OS again', async ({ page }) => {
  await useLocale(page, 'en')
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.goto('/')

  await themeButton(page, 'Light').click()
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')

  await themeButton(page, 'System').click()
  const overridden = await page.evaluate(() => document.documentElement.hasAttribute('data-theme'))
  expect(overridden, 'system clears the data-theme override').toBe(false)
  expect(await bodyBackground(page)).toBe(NIGHT)
})

test('the inline bootstrap restores a stored choice on reload, with no CSP violation', async ({
  page,
}) => {
  const cspErrors: string[] = []
  page.on('console', (message) => {
    if (message.type() === 'error' && /content security policy/i.test(message.text())) {
      cspErrors.push(message.text())
    }
  })

  await useLocale(page, 'en')
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.goto('/')
  await themeButton(page, 'Light').click()
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')

  await page.reload()
  // Set by the inline bootstrap before the bundle runs. If the CSP had blocked
  // that script, the stored choice would not be applied here at all.
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')
  expect(await bodyBackground(page)).toBe(PAGE)
  expect(cspErrors, 'the inline bootstrap must not trip the content security policy').toEqual([])
})
