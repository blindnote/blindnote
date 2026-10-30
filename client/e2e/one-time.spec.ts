import { expect, test } from '@playwright/test'
import { composeNote, useLocale } from './helpers'

test.beforeEach(async ({ page }) => {
  await useLocale(page, 'en')
})

test('a link opens exactly once, then reads as not found', async ({ page }) => {
  const secret = 'only once, then gone'
  const link = await composeNote(page, secret)

  await page.goto(link)
  await page.getByRole('button', { name: 'Break the Seal and Read' }).first().click()
  await expect(page.getByRole('region', { name: 'Secret message' })).toContainText(secret)

  // A second visit must not reveal that the note ever existed.
  await page.goto(link)
  await page.getByRole('button', { name: 'Break the Seal and Read' }).first().click()
  await expect(page.getByText("This message isn't here.")).toBeVisible()
})
