/**
 * This build's version and what the release server says about newer ones. The
 * startup check runs once per session and stays silent unless there is something to install.
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { checkForUpdate, getAppVersion } from '@/api'
import type { UpdateCheck } from '@/types'

export const useUpdateStore = defineStore('update', () => {
  const version = ref<string | null>(null)
  const latest = ref<UpdateCheck | null>(null)
  const checking = ref(false)
  const promptOpen = ref(false)
  let startupChecked = false

  async function loadVersion(): Promise<string> {
    version.value ??= await getAppVersion()
    return version.value
  }

  async function check(): Promise<UpdateCheck> {
    checking.value = true
    try {
      latest.value = await checkForUpdate()
      version.value = latest.value.current_version
      return latest.value
    } finally {
      checking.value = false
    }
  }

  /** A development build carries the placeholder version, so it would always look outdated. */
  async function checkAtStartup(): Promise<void> {
    if (startupChecked || import.meta.env.DEV) return
    startupChecked = true
    try {
      promptOpen.value = (await check()).update_available
    } catch {
      // Offline or rate-limited: the next start asks again, and Settings can ask by hand.
    }
  }

  function dismiss(): void {
    promptOpen.value = false
  }

  return { version, latest, checking, promptOpen, loadVersion, check, checkAtStartup, dismiss }
})
