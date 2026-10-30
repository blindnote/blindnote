import { expect, test } from '@playwright/test'
import { useLocale } from './helpers'

test('the trust badges read honestly in English', async ({ page }) => {
  await useLocale(page, 'en')
  await page.goto('/')
  await expect(page.getByText('One-Time Read').first()).toBeVisible()
  await expect(page.getByText('Zero-Knowledge').first()).toBeVisible()
  await expect(page.getByText('End-to-End Encrypted', { exact: true }).first()).toBeVisible()
  await expect(page.locator('main textarea')).toHaveAttribute('placeholder', 'For their eyes only…')
})

test('the Turkish zero-knowledge badge states the plain consequence', async ({ page }) => {
  await useLocale(page, 'tr')
  await page.goto('/')
  await expect(page.getByText('Biz Bile Göremeyiz').first()).toBeVisible()
  await expect(page.getByText('Sıfır Bilgi')).toHaveCount(0)
  await expect(page.locator('main textarea')).toHaveAttribute('placeholder', 'Yalnızca onun gözleri için…')
})
