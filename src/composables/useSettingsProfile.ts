import { computed, inject, provide, ref, type InjectionKey } from 'vue'
import { useConfigStore } from '@/stores/config'
import type { UpsertConfigInput } from '@/types'
import { SECTION_FIELDS, useProfileForm, type ProfileSection } from '@/composables/useProfileForm'

/** The settings pages that edit the professional profile, and the sections each shows. */
export type ProfilePage = 'profile' | 'invoicing'

export const PAGE_SECTIONS: Readonly<Record<ProfilePage, readonly ProfileSection[]>> = {
  profile: ['identity', 'profession', 'studio', 'payment'],
  invoicing: ['tax', 'numbering', 'invoice'],
}

const ALL_SECTIONS = [...PAGE_SECTIONS.profile, ...PAGE_SECTIONS.invoicing]

/** Where a save failed: the page whose fields need correcting, or none. */
export type SaveOutcome = { ok: true } | { ok: false; page: ProfilePage | null; error: unknown }

function snapshot(value: UpsertConfigInput): string {
  return JSON.stringify(value)
}

/**
 * One copy of the profile form shared by every settings page that edits it, so
 * moving between "Profilo" and "Fatturazione" keeps unsaved changes and a single
 * bar saves both.
 */
function createSettingsProfile() {
  const configStore = useConfigStore()
  const { form, errors, check, checkSections } = useProfileForm()
  const saving = ref(false)
  const saved = ref('')

  function reset(): void {
    const config = configStore.config
    if (config === null) return
    const { id: _id, created_at: _created, updated_at: _updated, ...profile } = config
    Object.assign(form, profile)
    for (const key of Object.keys(errors) as (keyof typeof errors)[]) delete errors[key]
    saved.value = snapshot(form)
  }

  async function load(): Promise<void> {
    if (configStore.config === null) await configStore.loadConfig()
    if (saved.value === '') reset()
  }

  const dirty = computed(() => saved.value !== '' && snapshot(form) !== saved.value)

  function firstInvalidPage(): ProfilePage | null {
    const pages = Object.keys(PAGE_SECTIONS) as ProfilePage[]
    return pages.find((page) => PAGE_SECTIONS[page].some((section) => SECTION_FIELDS[section].some((field) => errors[field] !== undefined))) ?? null
  }

  async function save(): Promise<SaveOutcome> {
    if (!checkSections(ALL_SECTIONS)) return { ok: false, page: firstInvalidPage(), error: 'Alcuni campi vanno corretti prima di salvare.' }
    saving.value = true
    try {
      await configStore.saveConfig({ ...form })
      saved.value = snapshot(form)
      return { ok: true }
    } catch (error) {
      return { ok: false, page: null, error }
    } finally {
      saving.value = false
    }
  }

  return { form, errors, check, saving, dirty, load, reset, save }
}

export type SettingsProfile = ReturnType<typeof createSettingsProfile>

const KEY: InjectionKey<SettingsProfile> = Symbol('settings-profile')

export function provideSettingsProfile(): SettingsProfile {
  const profile = createSettingsProfile()
  provide(KEY, profile)
  return profile
}

export function useSettingsProfile(): SettingsProfile {
  const profile = inject(KEY)
  if (profile === undefined) throw new Error('useSettingsProfile is only available under the settings layout')
  return profile
}
