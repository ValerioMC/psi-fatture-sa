<script setup lang="ts">
/**
 * Where the Sistema TS credentials come from, in the four steps a
 * professional actually takes: sign in on sistemats.it, print the
 * credentials, copy them here, verify. Open by default until they are set.
 */
import { ref, watch } from 'vue'
import { ChevronDown, ExternalLink, LifeBuoy } from 'lucide-vue-next'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useToastStore } from '@/stores/toast'

const SISTEMA_TS_PORTAL = 'https://sistemats1.sanita.finanze.it/portale/'

const props = defineProps<{ initiallyOpen: boolean }>()

const toast = useToastStore()
const open = ref(props.initiallyOpen)
watch(() => props.initiallyOpen, (value) => { if (value) open.value = true })

const STEPS = [
  {
    title: 'Entra nell’area riservata di sistemats.it',
    text: 'Con SPID, CIE oppure con codice fiscale e password.',
  },
  {
    title: 'Apri Profilo utente → Stampa credenziali',
    text: 'In alto a destra. Lì trovi la password e il PINCODE per la trasmissione dei dati.',
  },
  {
    title: 'Copiali qui sotto',
    text: 'Il codice fiscale è l’utente. Password e PINCODE restano solo nel portachiavi di questo computer.',
  },
  {
    title: 'Premi “Verifica credenziali”',
    text: 'Una chiamata di prova al Sistema TS conferma che sono giuste, senza inviare nulla.',
  },
] as const

async function openPortal(): Promise<void> {
  try {
    await openUrl(SISTEMA_TS_PORTAL)
  } catch (error) {
    toast.notifyError(error, 'Non riesco ad aprire il browser')
  }
}
</script>

<template>
  <div class="rounded-control border border-border bg-surface">
    <button
      type="button"
      class="flex w-full items-center gap-2 rounded-control px-3 py-2.5 text-left text-sm font-medium text-text hover:bg-surface-hover focus-ring"
      :aria-expanded="open"
      aria-controls="ts-credentials-guide"
      @click="open = !open"
    >
      <LifeBuoy :size="15" :stroke-width="1.8" class="shrink-0 text-accent" aria-hidden="true" />
      <span class="flex-1">Dove trovo queste credenziali?</span>
      <ChevronDown :size="15" class="shrink-0 text-text-subtle transition-transform" :class="open ? 'rotate-180' : ''" aria-hidden="true" />
    </button>

    <Transition name="pane">
      <div v-if="open" id="ts-credentials-guide" class="border-t border-border px-3 pt-3 pb-3.5">
        <ol class="space-y-3">
          <li v-for="(step, index) in STEPS" :key="step.title" class="flex gap-3">
            <span class="step-number" aria-hidden="true">{{ index + 1 }}</span>
            <div class="min-w-0">
              <p class="text-sm font-medium text-text">{{ step.title }}</p>
              <p class="mt-0.5 text-xs leading-relaxed text-text-muted">{{ step.text }}</p>
              <button
                v-if="index === 0"
                type="button"
                class="mt-1.5 inline-flex items-center gap-1 rounded-sm text-xs font-medium text-accent hover:underline focus-ring"
                @click="openPortal"
              >
                Apri sistemats.it <ExternalLink :size="12" aria-hidden="true" />
              </button>
            </div>
          </li>
        </ol>

        <div class="mt-3.5 space-y-2 border-t border-border pt-3 text-xs leading-relaxed text-text-subtle">
          <p>
            <span class="font-medium text-text-muted">Non hai ancora le credenziali?</span>
            Richiedile dal portale nell’area dei professionisti sanitari: dopo la verifica dell’iscrizione all’albo arrivano per PEC.
          </p>
          <p>
            <span class="font-medium text-text-muted">La verifica dice “password non valida”?</span>
            Entra una volta nel portale con codice fiscale e password: può chiederti di rinnovarla. Poi inserisci qui quella nuova.
          </p>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Numbered like a printed procedure: the order is the point, so the number is the first thing seen. */
.step-number {
  display: grid;
  place-items: center;
  flex-shrink: 0;
  width: 1.375rem;
  height: 1.375rem;
  margin-top: 0.0625rem;
  border-radius: 999px;
  font-size: var(--text-2xs);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--accent);
  background: var(--color-accent-soft);
  box-shadow: inset 0 0 0 1px var(--color-accent-line);
}
</style>
