<script setup lang="ts">
/**
 * The professional profile and the app's appearance. Appearance applies at
 * once; the profile is saved on demand, and a bar rises from the bottom as
 * soon as something differs from what is saved.
 */
import { computed, onMounted, ref } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { Monitor, Moon, Sun } from 'lucide-vue-next'
import { useConfigStore } from '@/stores/config'
import { useToastStore } from '@/stores/toast'
import type { UpsertConfigInput } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import FormSection from '@/components/ui/FormSection.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import ProfileSections from '@/components/profile/ProfileSections.vue'
import { useProfileForm, type ProfileSection } from '@/composables/useProfileForm'
import { useTheme } from '@/composables/useTheme'
import type { ThemePreference } from '@/utils/theme'

const configStore = useConfigStore()
const toast = useToastStore()
const { preference, setPreference } = useTheme()
const { form, errors, check, checkSections } = useProfileForm()

const ALL_SECTIONS: ProfileSection[] = ['identity', 'profession', 'tax', 'numbering', 'studio', 'payment']
const saving = ref(false)
const saved = ref('')

function snapshot(value: UpsertConfigInput): string {
  return JSON.stringify(value)
}

function fromConfig(): UpsertConfigInput | null {
  const config = configStore.config
  if (config === null) return null
  const { id: _id, created_at: _created, updated_at: _updated, ...profile } = config
  return profile
}

function reset(): void {
  const profile = fromConfig()
  if (profile === null) return
  Object.assign(form, profile)
  for (const key of Object.keys(errors) as (keyof typeof errors)[]) delete errors[key]
  saved.value = snapshot(form)
}

onMounted(async () => {
  if (configStore.config === null) await configStore.loadConfig()
  reset()
})

const dirty = computed(() => saved.value !== '' && snapshot(form) !== saved.value)

async function save(): Promise<void> {
  if (!checkSections(ALL_SECTIONS)) {
    toast.notifyError('Alcuni campi vanno corretti prima di salvare.')
    return
  }
  saving.value = true
  try {
    await configStore.saveConfig({ ...form })
    saved.value = snapshot(form)
    toast.notify('Profilo salvato')
  } catch (error) {
    toast.notifyError(error, 'Salvataggio non riuscito')
  } finally {
    saving.value = false
  }
}

const THEMES: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: 'system', label: 'Come il sistema', icon: Monitor },
  { value: 'light', label: 'Chiaro', icon: Sun },
  { value: 'dark', label: 'Scuro', icon: Moon },
]

// Leaving with unsaved changes asks first, instead of losing them silently.
const leaveOpen = ref(false)
let pendingLeave: (() => void) | null = null
onBeforeRouteLeave((_to, _from, next) => {
  if (!dirty.value) return next()
  pendingLeave = () => next()
  leaveOpen.value = true
  next(false)
})

function discardAndLeave(): void {
  leaveOpen.value = false
  reset()
  pendingLeave?.()
}
</script>

<template>
  <div>
    <PageHeader title="Impostazioni" subtitle="Il tuo profilo professionale e l'aspetto dell'app." />

    <div class="mx-auto max-w-[60rem] px-8 pt-6 pb-28">
      <AppCard class="settle mb-5 px-8 py-4">
        <FormSection title="Aspetto" description="Chiaro di giorno, scuro la sera, oppure segui il Mac o il PC.">
          <div class="grid grid-cols-3 gap-3" role="radiogroup" aria-label="Tema">
            <button
              v-for="theme in THEMES"
              :key="theme.value"
              type="button"
              role="radio"
              :aria-checked="preference === theme.value"
              class="flex flex-col items-center gap-2 rounded-card border px-3 py-4 text-sm transition-colors focus-ring"
              :class="preference === theme.value ? 'border-accent bg-accent-soft font-medium text-text' : 'border-border text-text-muted hover:border-border-strong hover:text-text'"
              @click="setPreference(theme.value)"
            >
              <component :is="theme.icon" :size="20" :stroke-width="1.6" :class="preference === theme.value ? 'text-accent' : ''" aria-hidden="true" />
              {{ theme.label }}
            </button>
          </div>
        </FormSection>
      </AppCard>

      <AppCard class="settle px-8 py-4" style="--settle: 1">
        <form id="profile-form" novalidate @submit.prevent="save">
          <ProfileSections :form="form" :errors="errors" :sections="ALL_SECTIONS" @check="check" />
        </form>
      </AppCard>
    </div>

    <Teleport to="body">
      <Transition name="rise">
        <div
          v-if="dirty"
          class="fixed bottom-6 left-[calc(50%+7.5rem)] z-(--z-overlay) flex -translate-x-1/2 items-center gap-4 rounded-card border border-border bg-surface-raised py-2 pl-5 pr-2 shadow-modal"
          role="region"
          aria-label="Modifiche non salvate"
        >
          <span class="whitespace-nowrap text-base text-text">Modifiche non salvate</span>
          <AppButton variant="ghost" size="sm" @click="reset">Annulla</AppButton>
          <AppButton variant="primary" size="sm" type="submit" form="profile-form" :loading="saving">Salva</AppButton>
        </div>
      </Transition>
    </Teleport>

    <ConfirmDialog
      :open="leaveOpen"
      title="Uscire senza salvare?"
      blast-radius="Le modifiche al profilo andranno perse."
      confirm-label="Esci senza salvare"
      @confirm="discardAndLeave"
      @cancel="leaveOpen = false"
    />
  </div>
</template>
