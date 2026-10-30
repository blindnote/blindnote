import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

// WCAG 2.1 contrast, checked against the live palette in styles/app.css rather
// than a copy that could rot: the test parses the tokens the stylesheet ships,
// so lowering one below its threshold fails here. AA is 4.5:1 for normal text
// and 3:1 for large text (>=24px, or >=18.66px bold) and UI graphics (1.4.3 /
// 1.4.11). Purely decorative borders (the card edge, which also has a fill
// difference) are exempt and not listed.

const css = readFileSync(new URL('../styles/app.css', import.meta.url), 'utf8')

// The first value of each --color-NAME is its @theme (light) value; the
// per-scheme override blocks that follow re-declare a few for dark.
function lightTokens(): Record<string, string> {
  const out: Record<string, string> = {}
  for (const match of css.matchAll(/--color-([\w-]+):\s*(#[0-9a-fA-F]{6})/g)) {
    const [, name, value] = match
    if (name && value && !(name in out)) out[name] = value.toLowerCase()
  }
  return out
}

function darkOverride(name: string, fallback: string): string {
  const re = new RegExp(`\\[data-theme="dark"\\][^}]*--color-${name}:\\s*(#[0-9a-fA-F]{6})`)
  return (css.match(re)?.[1] ?? fallback).toLowerCase()
}

const L = lightTokens()
const D = { ...L, muted: darkOverride('muted', L.muted ?? '#000000') }

function luminance(hex: string): number {
  const n = hex.replace('#', '')
  const channel = (i: number) => {
    const c = Number.parseInt(n.slice(i, i + 2), 16) / 255
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
  }
  return 0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4)
}

function hex(palette: Record<string, string>, name: string): string {
  const value = palette[name]
  if (!value) throw new Error(`unknown colour token: --color-${name}`)
  return value
}

function ratio(palette: Record<string, string>, fg: string, bg: string): number {
  const [l1, l2] = [luminance(hex(palette, fg)), luminance(hex(palette, bg))]
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05)
}

// [foreground, background, minimum ratio, usage].
type Pair = [string, string, number, string]

const lightPairs: Pair[] = [
  ['ink', 'page', 4.5, 'body text on the page'],
  ['ink', 'cream', 3, 'headings on a cream section'],
  ['ink-strong', 'page', 4.5, 'note content / links on the page'],
  ['ink-strong', 'cream', 4.5, 'note content on cream'],
  ['fur', 'page', 4.5, 'secondary text on the page'],
  ['fur', 'cream', 4.5, 'secondary text on cream'],
  ['muted', 'page', 4.5, 'placeholder / caption on the page'],
  ['muted', 'cream', 4.5, 'placeholder / caption on cream'],
  ['cream', 'ink', 4.5, 'label on the primary button'],
  ['cream', 'ink-hover', 4.5, 'label on the primary button, hover'],
  ['ink', 'accent', 4.5, 'ink on an accent chip / the key ticket'],
  ['danger', 'danger-soft', 4.5, 'danger text on its soft background'],
  ['danger', 'page', 4.5, 'danger text on the page'],
]

const darkPairs: Pair[] = [
  ['sand', 'night', 3, 'headings on the page'],
  ['sand-muted', 'night', 4.5, 'body text on the page'],
  ['sand-muted', 'night-raised', 4.5, 'body text on a raised section'],
  ['sand-muted', 'night-card', 4.5, 'body text on a card'],
  ['sand', 'night-card', 4.5, 'strong text on a card'],
  ['sand', 'night-footer', 4.5, 'footer text'],
  ['muted', 'night', 4.5, 'placeholder / caption on the page'],
  ['muted', 'night-card', 4.5, 'placeholder / caption on a card'],
  ['ink', 'accent', 4.5, 'ink on the accent primary button / key ticket'],
  ['accent', 'night', 4.5, 'accent text on the page'],
  ['danger-dark', 'night', 4.5, 'danger text on the page'],
  ['danger-dark', 'night-card', 4.5, 'danger text on a card'],
]

describe('WCAG AA contrast — light scheme', () => {
  it.each(lightPairs)('%s on %s >= %f:1  (%s)', (fg, bg, min) => {
    expect(ratio(L, fg, bg)).toBeGreaterThanOrEqual(min)
  })
})

describe('WCAG AA contrast — dark scheme', () => {
  it.each(darkPairs)('%s on %s >= %f:1  (%s)', (fg, bg, min) => {
    expect(ratio(D, fg, bg)).toBeGreaterThanOrEqual(min)
  })
})
