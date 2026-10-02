<script setup lang="ts">
/**
 * Emailing invoices to patients: the mailbox they leave from, how it works in
 * three steps, and the template every email starts from.
 */
import { onMounted } from 'vue'
import { Send } from 'lucide-vue-next'
import { useEmailStore } from '@/stores/email'
import { useToastStore } from '@/stores/toast'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import EmailAccountCard from '@/components/email/EmailAccountCard.vue'
import EmailTemplateCard from '@/components/email/EmailTemplateCard.vue'

const email = useEmailStore()
const toast = useToastStore()

onMounted(async () => {
  try {
    if (!email.loaded) await email.load()
  } catch (error) {
    toast.notifyError(error, 'Impostazioni email non leggibili')
  }
})

const STEPS = [
  { title: 'Scegli la casella', text: 'La PEC psypec.it che ti dà l’Ordine va benissimo: basta l’indirizzo e la sua password.' },
  { title: 'Verifica il collegamento', text: 'Un accesso di prova al server di posta, senza inviare nulla a nessuno.' },
  { title: 'Invia dalla fattura', text: 'Da ogni fattura emessa, “Invia al paziente”: parte con il PDF allegato e resta segnata.' },
] as const
</script>

<template>
  <div class="space-y-5">
    <div class="grid grid-cols-1 items-start gap-5 @min-[72rem]/settings:grid-cols-[minmax(0,1fr)_20rem]">
      <EmailAccountCard class="settle min-w-0" />
      <AppCard :padded="false" class="settle @min-[72rem]/settings:sticky @min-[72rem]/settings:top-6" style="--settle: 1">
        <CardHeader title="Come funziona" subtitle="Tre passi, una volta sola" :icon="Send" />
        <ol class="space-y-4 px-5 pb-5">
          <li v-for="(step, index) in STEPS" :key="step.title" class="flex gap-3">
            <span class="step-number tabular grid size-6 shrink-0 place-items-center rounded-full text-xs font-semibold text-accent" aria-hidden="true">{{ index + 1 }}</span>
            <div class="min-w-0">
              <p class="text-sm font-medium text-text">{{ step.title }}</p>
              <p class="mt-0.5 text-xs leading-relaxed text-text-muted">{{ step.text }}</p>
            </div>
          </li>
        </ol>
      </AppCard>
    </div>
    <EmailTemplateCard class="settle" style="--settle: 2" />
  </div>
</template>

<style scoped>
.step-number {
  background: var(--color-accent-soft);
  box-shadow: inset 0 0 0 1px var(--color-accent-line);
}
</style>
