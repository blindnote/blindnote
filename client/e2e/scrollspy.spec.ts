import { expect, test } from '@playwright/test'
import { useLocale } from './helpers'

// D-04: the desktop nav marks the section under the header with aria-current.
test('the nav highlights the section currently in view', async ({ page }) => {
  await useLocale(page, 'en')
  await page.goto('/')

  const nav = page.locator('header')
  const encryption = nav.getByRole('link', { name: 'End-to-End Encryption' })
  const faq = nav.getByRole('link', { name: 'FAQ' })

  await expect(encryption).not.toHaveAttribute('aria-current', 'true')

  await page.locator('#sifreleme').scrollIntoViewIfNeeded()
  await expect(encryption).toHaveAttribute('aria-current', 'true')

  await page.locator('#sss').scrollIntoViewIfNeeded()
  await expect(faq).toHaveAttribute('aria-current', 'true')
  await expect(encryption).not.toHaveAttribute('aria-current', 'true')
})
