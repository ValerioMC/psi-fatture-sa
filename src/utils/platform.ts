/**
 * Platform facts the interface adapts to: the modifier key shown in shortcut
 * hints, and whether the macOS title bar overlays the window content.
 */

export function isMac(userAgent: string = navigator.userAgent): boolean {
  return /Mac|iPhone|iPad/.test(userAgent)
}

/** "⌘K" on macOS, "Ctrl K" elsewhere. */
export function shortcutLabel(key: string, userAgent?: string): string {
  return isMac(userAgent) ? `⌘${key}` : `Ctrl ${key}`
}

/** True for the platform's primary shortcut modifier (⌘ on macOS, Ctrl elsewhere). */
export function hasPrimaryModifier(event: Pick<KeyboardEvent, 'metaKey' | 'ctrlKey'>, userAgent?: string): boolean {
  return isMac(userAgent) ? event.metaKey : event.ctrlKey
}

/**
 * Marks the document so CSS can make room for the macOS traffic lights when
 * the app runs inside Tauri with the overlay title bar.
 */
export function markPlatform(): void {
  const insideTauri = '__TAURI_INTERNALS__' in window
  if (insideTauri && isMac()) document.documentElement.dataset.titlebar = 'overlay'
}
