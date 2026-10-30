import { expect, test } from '@playwright/test'
import { composeNote, useLocale } from './helpers'

test.beforeEach(async ({ page }) => {
  await useLocale(page, 'en')
})

// D-05: the envelope and the button both reveal. Reading burns the note, so a
// double trigger must still send exactly one burn -- the ref guard in App.
test('triggering the envelope and the button together burns the note once', async ({ page }) => {
  const secret = 'burn me exactly once'
  const link = await composeNote(page, secret)

  const reveals: string[] = []
  page.on('request', (r) => {
    if (r.url().includes('/api/reveal')) reveals.push(r.method())
  })

  await page.goto(link)
  await expect(page.getByRole('heading', { name: 'Knock Knock' })).toBeVisible()

  // Fire both synchronously, before React can disable either.
  await page.evaluate(() => {
    const label = 'Break the Seal and Read'
    const buttons = Array.from(document.querySelectorAll('button'))
    const envelope = buttons.find((b) => b.getAttribute('aria-label') === label)
    const cta = buttons.find((b) => b !== envelope && (b.textContent ?? '').includes(label))
    ;(envelope as HTMLButtonElement | undefined)?.click()
    ;(cta as HTMLButtonElement | undefined)?.click()
  })

  await expect(page.getByRole('region', { name: 'Secret message' })).toContainText(secret)
  expect(reveals, 'a double trigger must burn only once').toHaveLength(1)
})
