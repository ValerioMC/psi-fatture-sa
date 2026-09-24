<script setup lang="ts">
/**
 * First run: three short steps instead of one long form. Each step is
 * validated before the next opens, so the last button cannot fail on a field
 * the user left two screens ago.
 */
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, ArrowRight, Check, HardDrive, Receipt, WifiOff } from 'lucide-vue-next'
import BrandMark from '@/components/ui/BrandMark.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ProfileSections from '@/components/profile/ProfileSections.vue'
import { useConfigStore } from '@/stores/config'
import { useToastStore } from '@/stores/toast'
import { useProfileForm, type ProfileSection } from '@/composables/useProfileForm'
import { useSmoothScroll } from '@/composables/useSmoothScroll'

const router = useRouter()
const configStore = useConfigStore()
const toast = useToastStore()
const { form, errors, check, checkSections } = useProfileForm()
useSmoothScroll()

const STEPS: { title: string; sections: ProfileSection[] }[] = [
  { title: 'Chi sei', sections: ['identity', 'profession'] },
  { title: 'Fisco', sections: ['tax', 'numbering'] },
  { title: 'Studio', sections: ['studio', 'payment'] },
]

const step = ref(0)
const saving = ref(false)
const current = computed(() => STEPS[step.value])
const isLast = computed(() => step.value === STEPS.length - 1)

function next(): void {
  if (!checkSections(current.value.sections)) return
  step.value += 1
  window.scrollTo({ top: 0 })
}

async function finish(): Promise<void> {
  if (!checkSections(current.value.sections)) return
  saving.value = true
  try {
    await configStore.saveConfig({ ...form })
    toast.notify(`Profilo pronto. Buon lavoro, ${form.first_name}.`)
    void router.push('/dashboard')
  } catch (error) {
    toast.notifyError(error, 'Salvataggio non riuscito')
  } finally {
    saving.value = false
  }
}

const PROMISES = [
  { icon: Receipt, text: 'ENPAP, marca da bollo e diciture di legge calcolate per te.' },
  { icon: HardDrive, text: 'Pazienti e fatture restano su questo computer.' },
  { icon: WifiOff, text: 'Funziona anche senza connessione.' },
]
</script>

<template>
  <div class="grid min-h-screen grid-cols-1 lg:grid-cols-[minmax(22rem,2fr)_3fr]">
    <!-- ── The promise ────────────────────────────────────────────────────── -->
    <aside class="relative hidden flex-col justify-between overflow-hidden border-r border-border bg-surface px-12 py-12 lg:flex" data-tauri-drag-region>
      <div class="sidebar-top flex items-center gap-2.5" data-tauri-drag-region>
        <BrandMark :size="30" />
        <span class="display text-xl text-text">PSI Fatture</span>
      </div>

      <div class="settle">
        <h1 class="display text-[2.75rem] leading-[1.08] text-text">Il tuo studio,<br /><em class="text-accent">in ordine.</em></h1>
        <p class="mt-4 max-w-sm text-md text-text-muted">Tre passi per preparare la carta intestata. Poi scrivi la prima fattura.</p>
        <ul class="mt-10 space-y-4">
          <li v-for="promise in PROMISES" :key="promise.text" class="flex items-start gap-3 text-base text-text-muted">
            <span class="grid size-8 shrink-0 place-items-center rounded-full bg-accent-soft text-accent"><component :is="promise.icon" :size="16" :stroke-width="1.75" aria-hidden="true" /></span>
            <span class="pt-1.5">{{ promise.text }}</span>
          </li>
        </ul>
      </div>

      <p class="text-xs text-text-subtle">Puoi cambiare tutto più tardi, da Impostazioni.</p>
      <!-- A large, faint Ψ: the room's watermark. -->
      <svg class="pointer-events-none absolute -right-32 -bottom-40 w-[24rem] text-accent opacity-[0.045]" viewBox="0 0 36 36" fill="none" aria-hidden="true">
        <g stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M11.5 10.5v4.2a6.5 6.5 0 0 0 13 0v-4.2" /><path d="M18 9.5v17" /></g>
      </svg>
    </aside>

    <!-- ── The steps ──────────────────────────────────────────────────────── -->
    <main class="flex flex-col px-8 py-12 lg:px-16">
      <ol class="mb-8 flex items-center gap-3" aria-label="Passaggi">
        <li v-for="(item, index) in STEPS" :key="item.title" class="flex items-center gap-3">
          <span
            class="grid size-7 place-items-center rounded-full text-sm font-medium transition-colors"
            :class="index < step ? 'bg-safe text-white' : index === step ? 'bg-accent text-accent-ink' : 'border border-border-strong text-text-subtle'"
            :aria-current="index === step ? 'step' : undefined"
          >
            <Check v-if="index < step" :size="14" :stroke-width="2.5" aria-hidden="true" />
            <template v-else>{{ index + 1 }}</template>
          </span>
          <span class="text-base" :class="index === step ? 'font-medium text-text' : 'text-text-subtle'">{{ item.title }}</span>
          <span v-if="index < STEPS.length - 1" class="h-px w-10 bg-border-strong" aria-hidden="true" />
        </li>
      </ol>

      <form class="max-w-[46rem] flex-1" novalidate @submit.prevent="isLast ? finish() : next()">
        <Transition name="pane" mode="out-in">
          <ProfileSections :key="step" :form="form" :errors="errors" :sections="current.sections" @check="check" />
        </Transition>

        <div class="mt-6 flex items-center gap-3 border-t border-border pt-6">
          <AppButton v-if="step > 0" variant="ghost" :icon="ArrowLeft" @click="step -= 1">Indietro</AppButton>
          <div class="flex-1" />
          <span class="text-sm text-text-subtle">{{ step + 1 }} di {{ STEPS.length }}</span>
          <AppButton v-if="!isLast" type="submit" variant="primary" :icon-right="ArrowRight">Avanti</AppButton>
          <AppButton v-else type="submit" variant="primary" :icon="Check" :loading="saving">Apri lo studio</AppButton>
        </div>
      </form>
    </main>
  </div>
</template>
