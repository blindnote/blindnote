import { useState } from 'react'

/**
 * The appearance choice. `system` follows the OS via the CSS
 * `prefers-color-scheme` media query (no `data-theme` set); `light`/`dark` pin
 * `data-theme` on `<html>` and override the system. The inline bootstrap in
 * `index.html` applies a stored `light`/`dark` before first paint so an
 * override never flashes the system theme first; this module is the React half
 * that reads and changes the choice at runtime.
 */
export type Theme = 'system' | 'light' | 'dark'

/** All choices, in the order the switcher shows them. */
export const THEMES: readonly Theme[] = ['system', 'light', 'dark']

/** The key the bootstrap and this module share. Keep them in step. */
const STORAGE_KEY = 'blindnote-theme'

function isTheme(value: string): value is Theme {
  return (THEMES as readonly string[]).includes(value)
}

/** The remembered choice, or `system` when none is stored or storage is blocked. */
function readStoredTheme(): Theme {
  try {
    const value = localStorage.getItem(STORAGE_KEY)
    return value && isTheme(value) ? value : 'system'
  } catch {
    return 'system'
  }
}

/**
 * Applies a theme now and remembers it. `system` removes the override so the
 * media query takes over again; `light`/`dark` set it. Storing is best-effort:
 * a blocked store just means the choice will not survive a reload.
 */
function applyTheme(theme: Theme): void {
  const root = document.documentElement
  if (theme === 'system') {
    root.removeAttribute('data-theme')
  } else {
    root.setAttribute('data-theme', theme)
  }
  try {
    localStorage.setItem(STORAGE_KEY, theme)
  } catch {
    // Private mode or blocked storage: the choice simply will not persist.
  }
}

/**
 * The active theme and a setter that applies it to `<html>` and remembers it.
 * The initial value matches what the bootstrap already applied, so the control
 * and the page agree from the first render.
 */
export function useTheme(): { theme: Theme; setTheme: (theme: Theme) => void } {
  const [theme, setThemeState] = useState<Theme>(() => readStoredTheme())

  const setTheme = (next: Theme) => {
    applyTheme(next)
    setThemeState(next)
  }

  return { theme, setTheme }
}
