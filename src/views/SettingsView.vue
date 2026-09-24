<script setup lang="ts">
/**
 * The professional profile and the app's appearance. Appearance applies at
 * once; the profile is saved on demand, and a bar rises from the bottom as
 * soon as something differs from what is saved.
 */
import { computed, onMounted, ref } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { FileText, Monitor, Moon, Palette, Settings, Sun } from 'lucide-vue-next'
import { useConfigStore } from '@/stores/config'
import { useToastStore } from '@/stores/toast'
import type { UpsertConfigInput } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import ProfileSections from '@/components/profile/ProfileSections.vue'
import { useProfileForm, type ProfileSection } from '@/composables/useProfileForm'
import { useTheme } from '@/composables/useTheme'
import type { ThemePreference } from '@/utils/theme'
import { TAX_REGIME_LABEL } from '@/utils/labels'

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

/** The invoice letterhead as it will print, redrawn live from the form. */
const letterhead = computed(() => {
  const place = [form.zip_code, form.city].filter(Boolean).join(' ')
  return {
    name: [form.title, form.first_name, form.last_name].filter(Boolean).join(' ').trim() || 'Il tuo nome',
    profession: form.profession === 'psicoterapeuta' ? 'Psicoterapeuta' : 'Psicologo',
    lines: [
      form.address,
      place + (form.province ? ` (${form.province})` : ''),
      form.vat_number ? `P.IVA ${form.vat_number}` : '',
      form.fiscal_code ? `C.F. ${form.fiscal_code}` : '',
      form.albo_number ? `Albo ${form.albo_region ?? ''} n. ${form.albo_number}`.replace(/\s+/g, ' ') : '',
    ].filter((line) => line && line.trim() !== ''),
    regime: form.tax_regime ? TAX_REGIME_LABEL[form.tax_regime] : '',
  }
})

const THEMES: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: 'system', label: 'Sistema', icon: Monitor },
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
    <PageHeader title="Impostazioni" subtitle="Il tuo profilo professionale e l'aspetto dell'app." :icon="Settings" />

    <div class="page grid grid-cols-1 items-start gap-5 pt-6 pb-28 xl:grid-cols-[minmax(0,1fr)_21rem] 2xl:grid-cols-[minmax(0,1fr)_24rem]">
      <AppCard class="settle min-w-0 px-8 py-4">
        <form id="profile-form" novalidate @submit.prevent="save">
          <ProfileSections :form="form" :errors="errors" :sections="ALL_SECTIONS" @check="check" />
        </form>
      </AppCard>

      <aside class="settle space-y-5 xl:sticky xl:top-32" style="--settle: 1">
        <!-- Appearance applies at once: each option is a tiny window in that theme. -->
        <AppCard :padded="false">
          <CardHeader title="Aspetto" subtitle="Si applica subito" :icon="Palette" />
          <div class="grid grid-cols-3 gap-2.5 px-5 pb-5" role="radiogroup" aria-label="Tema">
            <button
              v-for="theme in THEMES"
              :key="theme.value"
              type="button"
              role="radio"
              :aria-checked="preference === theme.value"
              class="group flex flex-col items-stretch gap-2 rounded-control p-1.5 text-xs transition-colors focus-ring"
              :class="preference === theme.value ? 'bg-accent-soft font-medium text-text ring-1 ring-accent' : 'text-text-muted ring-1 ring-border hover:ring-border-strong hover:text-text'"
              @click="setPreference(theme.value)"
            >
              <span class="theme-swatch" :data-swatch="theme.value" aria-hidden="true">
                <span class="theme-swatch-side" />
                <span class="theme-swatch-main"><span /><span /><span /></span>
              </span>
              <span class="flex items-center justify-center gap-1">
                <component :is="theme.icon" :size="12" :stroke-width="1.8" :class="preference === theme.value ? 'text-accent' : ''" aria-hidden="true" />
                {{ theme.label }}
              </span>
            </button>
          </div>
        </AppCard>

        <!-- The letterhead, as it will print: what the profile fields are for. -->
        <AppCard :padded="false">
          <CardHeader title="Intestazione in fattura" subtitle="Anteprima dal profilo" :icon="FileText" />
          <div class="px-5 pb-5">
            <div class="letterhead rounded-[10px] px-4 py-4">
              <p class="display text-lg leading-tight text-[#1d1b24]">{{ letterhead.name }}</p>
              <p class="mt-0.5 text-2xs font-medium tracking-[0.06em] text-[#3a3e9f] uppercase">{{ letterhead.profession }}</p>
              <div class="my-3 h-px bg-[#e4e0d7]" />
              <p v-for="line in letterhead.lines" :key="line" class="truncate text-2xs leading-[1.1rem] text-[#56525e]">{{ line }}</p>
              <p v-if="letterhead.regime" class="mt-2 inline-block rounded-full bg-[#edeae3] px-2 text-[10px] leading-4 text-[#56525e]">{{ letterhead.regime }}</p>
            </div>
          </div>
        </AppCard>
      </aside>
    </div>

    <Teleport to="body">
      <Transition name="rise">
        <div
          v-if="dirty"
          class="fixed bottom-6 left-[calc(50%+var(--spacing-sidebar)/2)] z-(--z-overlay) flex -translate-x-1/2 items-center gap-4 rounded-card border border-border bg-surface-raised py-2 pl-5 pr-2 shadow-modal"
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

<style scoped>
/* A theme option: a thumbnail window, sidebar and three content lines, in that theme's colours. */
.theme-swatch {
  display: flex;
  height: 3.25rem;
  overflow: hidden;
  border-radius: 6px;
  box-shadow: inset 0 0 0 1px rgb(0 0 0 / 0.08);
}
.theme-swatch-side { width: 30%; }
.theme-swatch-main { flex: 1; display: flex; flex-direction: column; gap: 4px; padding: 7px 6px; }
.theme-swatch-main > span { height: 4px; border-radius: 99px; }
.theme-swatch-main > span:nth-child(1) { width: 70%; }
.theme-swatch-main > span:nth-child(2) { width: 90%; }
.theme-swatch-main > span:nth-child(3) { width: 45%; }

[data-swatch='light'] .theme-swatch-side { background: #fbfaf7; box-shadow: inset -1px 0 0 #e4e0d7; }
[data-swatch='light'] .theme-swatch-main { background: #f5f3ee; }
[data-swatch='light'] .theme-swatch-main > span { background: #cfc9bc; }
[data-swatch='light'] .theme-swatch-main > span:first-child { background: #3a3e9f; }

[data-swatch='dark'] .theme-swatch-side { background: #17161d; box-shadow: inset -1px 0 0 #2a2834; }
[data-swatch='dark'] .theme-swatch-main { background: #121117; }
[data-swatch='dark'] .theme-swatch-main > span { background: #3b3947; }
[data-swatch='dark'] .theme-swatch-main > span:first-child { background: #a5a9ff; }

/* "Come il sistema" is literally half of each. */
[data-swatch='system'] { background: linear-gradient(135deg, #f5f3ee 50%, #121117 50%); }
[data-swatch='system'] .theme-swatch-side { background: linear-gradient(135deg, #fbfaf7 50%, #17161d 50%); }
[data-swatch='system'] .theme-swatch-main > span { background: #8b8799; }
[data-swatch='system'] .theme-swatch-main > span:first-child { background: #6e72d0; }

/* The letterhead is paper in both themes: it previews a printed page. */
.letterhead {
  background: #ffffff;
  box-shadow: 0 0 0 1px rgb(40 34 20 / 0.08), 0 6px 16px -8px rgb(40 34 20 / 0.25);
}
</style>
