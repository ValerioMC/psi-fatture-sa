import { nextTick, onBeforeUnmount, watch, type Ref } from 'vue'

const FOCUSABLE = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(',')

interface FocusTrapOptions {
  /** Called on Escape. */
  onEscape: () => void
  /** Element to focus on open; defaults to the first focusable element. */
  initialFocus?: () => HTMLElement | null | undefined
}

/**
 * Keeps keyboard focus inside a dialog while it is open, closes it on Escape,
 * and hands focus back to whatever opened it once it closes.
 */
export function useFocusTrap(
  open: Ref<boolean>,
  container: Ref<HTMLElement | null>,
  options: FocusTrapOptions,
): void {
  let returnFocusTo: HTMLElement | null = null

  function focusables(): HTMLElement[] {
    if (container.value === null) return []
    return Array.from(container.value.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
      (element) => element.offsetParent !== null || element === document.activeElement,
    )
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.stopPropagation()
      options.onEscape()
      return
    }
    if (event.key !== 'Tab') return
    const items = focusables()
    if (items.length === 0) {
      event.preventDefault()
      return
    }
    const first = items[0]
    const last = items[items.length - 1]
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault()
      last.focus()
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault()
      first.focus()
    }
  }

  function activate(): void {
    returnFocusTo = document.activeElement instanceof HTMLElement ? document.activeElement : null
    document.addEventListener('keydown', onKeydown, true)
    void nextTick(() => {
      const target = options.initialFocus?.() ?? focusables()[0] ?? container.value
      target?.focus({ preventScroll: true })
    })
  }

  function deactivate(): void {
    document.removeEventListener('keydown', onKeydown, true)
    returnFocusTo?.focus({ preventScroll: true })
    returnFocusTo = null
  }

  watch(open, (isOpen, wasOpen) => {
    if (isOpen && !wasOpen) activate()
    else if (!isOpen && wasOpen) deactivate()
  }, { immediate: true })

  onBeforeUnmount(() => {
    if (open.value) deactivate()
  })
}
