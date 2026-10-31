import { expect, test } from '@playwright/test'

// The header's language switcher is for the first impression at the top of the
// page; once the header elevates on scroll it steps out of the way, leaving the
// nav and the create button. The footer still carries a switcher. This guards
// that hide-on-scroll against the shipped binary. The header's only `group` is
// the switcher (the mobile one is display:none at this width), so counting it
// is enough.

test('the header language switcher hides once the header elevates on scroll', async ({ page }) => {
  await page.goto('/')
  const switcher = page.locator('header').getByRole('group')
  await expect(switcher).toBeVisible()

  await page.evaluate(() => window.scrollTo(0, 1200))
  await expect(switcher).toHaveCount(0)
})
