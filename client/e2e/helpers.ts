import { expect, type Page } from '@playwright/test'

/**
 * Pins the interface language so copy assertions are deterministic. Detection
 * from `navigator.languages` is skipped once a choice is stored.
 */
export async function useLocale(page: Page, locale: 'en' | 'tr'): Promise<void> {
  await page.addInitScript((value) => {
    try {
      localStorage.setItem('blindnote.locale', value)
    } catch {
      /* private mode: detection then decides, which the tests set up for anyway */
    }
  }, locale)
}

/**
 * Composes a note through the UI (English) and returns the one-time link it
 * produced. The link is read from the share card, where it is the only text
 * carrying the `/#` of a fragment.
 */
export async function composeNote(page: Page, secret: string): Promise<string> {
  await page.goto('/')
  await page.locator('main textarea').fill(secret)
  await page.locator('main').getByRole('button', { name: 'Create a Secret Message' }).click()
  await expect(page.getByRole('heading', { name: 'Your Link Is Ready!' })).toBeVisible()

  const link = await page.locator('p[title*="/#"]').first().getAttribute('title')
  expect(link, 'the share card should show the link').toBeTruthy()
  return link as string
}

/** The note id: the part of the fragment before the `/`. */
export function idOf(link: string): string {
  return new URL(link).hash.slice(1).split('/')[0] ?? ''
}

/** The key: the part of the fragment after the `/`. */
export function keyOf(link: string): string {
  return new URL(link).hash.slice(1).split('/')[1] ?? ''
}
