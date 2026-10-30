import { expect, test } from '@playwright/test'
import { composeNote, idOf, keyOf, useLocale } from './helpers'

test.beforeEach(async ({ page }) => {
  await useLocale(page, 'en')
})

// CLAUDE.md s12.9, as an automated check rather than a one-off HAR scan: the
// key and the plaintext must not cross the wire, in any request, ever.
test('the key and the plaintext never leave the browser', async ({ page, baseURL }) => {
  const secret = 'super secret plaintext 9f3a-do-not-leak'
  const seen: { url: string; body: string | null; referer: string }[] = []
  page.on('request', (r) => {
    seen.push({ url: r.url(), body: r.postData(), referer: r.headers().referer ?? '' })
  })

  const link = await composeNote(page, secret)
  await page.goto(link)
  await page.getByRole('button', { name: 'Break the Seal and Read' }).first().click()
  await expect(page.getByRole('region', { name: 'Secret message' })).toContainText(secret)

  const key = keyOf(link)
  expect(key.length).toBe(43)

  for (const req of seen) {
    // Bodies and the Referer header are what actually travel; the top-level
    // navigation URL keeps its fragment in the browser but never sends it.
    expect(req.referer, `referer leaked the key: ${req.url}`).not.toContain(key)
    if (req.body) {
      expect(req.body, `key in body of ${req.url}`).not.toContain(key)
      expect(req.body, `plaintext in body of ${req.url}`).not.toContain(secret)
    }
    if (req.url.startsWith(`${baseURL}/api/`)) {
      expect(req.url, 'key in an api url').not.toContain(key)
      expect(req.url, 'plaintext in an api url').not.toContain(secret)
    }
  }
})

test('GET never burns a note: the reveal endpoint refuses it with 405', async ({ request }) => {
  const res = await request.get('/api/reveal')
  expect(res.status()).toBe(405)
})

test('a missing note and an already-read note are indistinguishable', async ({ page, request }) => {
  const link = await composeNote(page, 'one and done')
  const id = idOf(link)

  await page.goto(link)
  await page.getByRole('button', { name: 'Break the Seal and Read' }).first().click()
  await expect(page.getByRole('region', { name: 'Secret message' })).toBeVisible()

  const burned = await request.post('/api/reveal', { data: { id } })
  const missing = await request.post('/api/reveal', { data: { id: 'A'.repeat(22) } })

  expect(burned.status()).toBe(404)
  expect(missing.status()).toBe(404)
  expect(await burned.text()).toBe(await missing.text())
})

test('loading a link does not burn the note; only the click does', async ({ page }) => {
  const secret = 'click to burn'
  const link = await composeNote(page, secret)

  const reveals: string[] = []
  page.on('request', (r) => {
    if (r.url().includes('/api/reveal')) reveals.push(r.method())
  })

  await page.goto(link)
  await expect(page.getByRole('heading', { name: 'Knock Knock' })).toBeVisible()
  await page.waitForTimeout(300)
  expect(reveals, 'no reveal before the click').toHaveLength(0)

  await page.getByRole('button', { name: 'Break the Seal and Read' }).first().click()
  await expect(page.getByRole('region', { name: 'Secret message' })).toContainText(secret)
  expect(reveals, 'exactly one reveal, on the click').toHaveLength(1)
})
