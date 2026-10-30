import { expect, test } from '@playwright/test'
import { composeNote, useLocale } from './helpers'

test.beforeEach(async ({ page }) => {
  await useLocale(page, 'en')
})

test('a note is composed, linked, opened and read', async ({ page }) => {
  const secret = 'the vault code is 4815-1623-correct-horse'
  const link = await composeNote(page, secret)
  expect(link).toContain('/#')

  await page.goto(link)
  await expect(page.getByRole('heading', { name: 'Knock Knock' })).toBeVisible()

  await page.getByRole('button', { name: 'Break the Seal and Read' }).first().click()

  await expect(page.getByRole('heading', { name: 'You can view this message only once!' })).toBeVisible()
  await expect(page.getByRole('region', { name: 'Secret message' })).toContainText(secret)
})
