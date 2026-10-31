import { expect, test } from '@playwright/test'
import { composeNote, useLocale } from './helpers'

// Copying is the whole point of the link-ready screen, and copying the opened
// message is the main action on the read screen. These drive both against the
// real clipboard (permissions granted) so a broken copy cannot pass silently --
// the sender has to get the exact link out, and the reader the exact text.

test.beforeEach(async ({ page, context }) => {
  await useLocale(page, 'en')
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
})

test('copying the link puts the full one-time URL on the clipboard', async ({ page }) => {
  const link = await composeNote(page, 'the link I need to share, in full')
  await page.getByRole('button', { name: 'Copy', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Copied', exact: true })).toBeVisible()
  const clip = await page.evaluate(() => navigator.clipboard.readText())
  expect(clip).toBe(link)
})

test('copying the opened message puts its text on the clipboard', async ({ page }) => {
  const secret = 'the message body the reader copies out, verbatim'
  const link = await composeNote(page, secret)
  await page.goto(link)
  await page.getByRole('button', { name: 'Break the Seal and Read' }).first().click()
  await expect(page.getByRole('heading', { name: 'You can view this message only once!' })).toBeVisible()
  await page.getByRole('button', { name: 'Copy text' }).click()
  const clip = await page.evaluate(() => navigator.clipboard.readText())
  expect(clip).toBe(secret)
})
