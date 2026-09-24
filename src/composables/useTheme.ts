import { computed, ref, type ComputedRef, type Ref } from 'vue'
import {
  readThemePreference,
  resolveTheme,
  writeThemePreference,
  type ResolvedTheme,
  type ThemePreference,
} from '@/utils/theme'

const preference = ref<ThemePreference>('system')
const systemPrefersDark = ref(false)
let initialised = false

function safeStorage(): Storage | undefined {
  try {
    return window.localStorage
  } catch {
    return undefined
  }
}

function apply(): void {
  document.documentElement.dataset.theme = resolveTheme(preference.value, systemPrefersDark.value)
}

/**
 * Applies the stored appearance before the app mounts, so the first frame is
 * already in the right theme, and follows the OS while the preference is "system".
 */
export function initTheme(): void {
  if (initialised) return
  initialised = true
  preference.value = readThemePreference(safeStorage())
  const media = window.matchMedia('(prefers-color-scheme: dark)')
  systemPrefersDark.value = media.matches
  media.addEventListener('change', (event) => {
    systemPrefersDark.value = event.matches
    apply()
  })
  apply()
}

export function useTheme(): {
  preference: Ref<ThemePreference>
  resolved: ComputedRef<ResolvedTheme>
  setPreference: (next: ThemePreference) => void
} {
  const resolved = computed(() => resolveTheme(preference.value, systemPrefersDark.value))

  function setPreference(next: ThemePreference): void {
    preference.value = next
    writeThemePreference(safeStorage(), next)
    apply()
  }

  return { preference, resolved, setPreference }
}
