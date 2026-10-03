/**
 * This build's version and what the release server says about newer ones. The
 * startup check runs once per session and stays silent unless there is something to install.
 * On a Mac the app installs the update itself, so the new version opens without `xattr`.
 */
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { checkForUpdate, getAppVersion, installUpdate } from '@/api'
import { errorMessage } from '@/stores/toast'
import type { Platform, UpdateCheck } from '@/types'

const SELF_INSTALLING: readonly Platform[] = ['mac_os_arm64', 'mac_os_x64']

export const useUpdateStore = defineStore('update', () => {
  const version = ref<string | null>(null)
  const latest = ref<UpdateCheck | null>(null)
  const checking = ref(false)
  const installing = ref(false)
  const installFailure = ref<string | null>(null)
  const promptOpen = ref(false)
  let startupChecked = false

  /** False after a failed install too: the browser download is the way out. */
  const installsItself = computed(
    () => latest.value !== null && SELF_INSTALLING.includes(latest.value.platform) && installFailure.value === null,
  )

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

  /** Installs and restarts on a Mac; elsewhere, or after a failure, opens the download in the browser. */
  async function apply(): Promise<void> {
    if (latest.value === null) return
    if (!installsItself.value) {
      await openUrl(latest.value.download_url)
      promptOpen.value = false
      return
    }
    installing.value = true
    try {
      await installUpdate()
    } catch (error) {
      installFailure.value = errorMessage(error)
    } finally {
      installing.value = false
    }
  }

  function dismiss(): void {
    if (!installing.value) promptOpen.value = false
  }

  return {
    version,
    latest,
    checking,
    installing,
    installFailure,
    installsItself,
    promptOpen,
    loadVersion,
    check,
    checkAtStartup,
    apply,
    dismiss,
  }
})
