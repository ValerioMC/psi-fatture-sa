import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ToastKind = 'success' | 'error' | 'info'

export interface ToastAction {
  label: string
  run: () => void | Promise<void>
}

export interface Toast {
  id: number
  kind: ToastKind
  message: string
  action?: ToastAction
}

/**
 * How long each kind stays on screen. Errors are read, successes only glanced
 * at; an action (e.g. "Annulla") buys extra time so it can be reached for.
 */
export const TOAST_DURATION_MS: Readonly<Record<ToastKind, number>> = {
  success: 3200,
  info: 4000,
  error: 7000,
}
export const TOAST_ACTION_BONUS_MS = 3000
const MAX_VISIBLE = 4

/** Database errors worth translating: the raw SQLite text means nothing to a psychologist. */
const KNOWN_ERRORS: readonly [RegExp, string][] = [
  [/FOREIGN KEY constraint failed/i, 'È collegato a fatture o appuntamenti, quindi non può essere eliminato.'],
  [/UNIQUE constraint failed: invoices/i, 'Esiste già una fattura con questo numero nello stesso anno.'],
]

/** Turns whatever a Tauri command rejected with into a sentence for the user. */
export function errorMessage(error: unknown): string {
  const raw = error instanceof Error ? error.message : typeof error === 'string' ? error : ''
  if (raw === '') return 'Si è verificato un errore imprevisto.'
  const known = KNOWN_ERRORS.find(([pattern]) => pattern.test(raw))
  return known ? known[1] : raw
}

export function toastDuration(kind: ToastKind, hasAction: boolean): number {
  return TOAST_DURATION_MS[kind] + (hasAction ? TOAST_ACTION_BONUS_MS : 0)
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([])
  const timers = new Map<number, ReturnType<typeof setTimeout>>()
  let nextId = 1

  function dismiss(id: number): void {
    const timer = timers.get(id)
    if (timer !== undefined) clearTimeout(timer)
    timers.delete(id)
    toasts.value = toasts.value.filter((toast) => toast.id !== id)
  }

  function push(kind: ToastKind, message: string, action?: ToastAction): number {
    const id = nextId++
    toasts.value = [...toasts.value, { id, kind, message, action }].slice(-MAX_VISIBLE)
    timers.set(id, setTimeout(() => dismiss(id), toastDuration(kind, action !== undefined)))
    return id
  }

  function notify(message: string, action?: ToastAction): number {
    return push('success', message, action)
  }

  function notifyInfo(message: string): number {
    return push('info', message)
  }

  function notifyError(error: unknown, prefix?: string): number {
    const detail = errorMessage(error)
    return push('error', prefix ? `${prefix}: ${detail}` : detail)
  }

  async function runAction(toast: Toast): Promise<void> {
    dismiss(toast.id)
    try {
      await toast.action?.run()
    } catch (error) {
      notifyError(error)
    }
  }

  return { toasts, notify, notifyInfo, notifyError, dismiss, runAction }
})
