<script setup lang="ts">
/**
 * One invoice and the patient's inbox: whether it went, when and to whom, the way
 * to send it (again), and every attempt. A draft is not sent: it has no fiscal value yet.
 */
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { ChevronDown, Send } from 'lucide-vue-next'
import { useEmailStore } from '@/stores/email'
import { useToastStore } from '@/stores/toast'
import type { Invoice } from '@/types'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import MailMark from '@/components/ui/MailMark.vue'
import SendInvoiceEmailDialog from './SendInvoiceEmailDialog.vue'
import { describeMailMark } from '@/utils/email'
import { formatUtcTimestamp } from '@/utils/format'

const props = defineProps<{ invoice: Invoice }>()

const email = useEmailStore()
const toast = useToastStore()
const dialogOpen = ref(false)
const historyOpen = ref(false)

onMounted(async () => {
  if (email.loaded) return
  try {
    await email.load()
  } catch (error) {
    toast.notifyError(error, 'Invii email non leggibili')
  }
})

const state = computed(() => email.stateOf(props.invoice.id))
const sendable = computed(() => props.invoice.status !== 'draft' && props.invoice.status !== 'cancelled')
const history = computed(() => email.emails.filter((record) => record.invoice_id === props.invoice.id))

const sentence = computed(() => {
  const { lastSent, last } = state.value
  if (lastSent) return `A ${lastSent.recipient}, ${formatUtcTimestamp(lastSent.sent_at)}.`
  if (last) return last.error ?? 'Il server non ha accettato il messaggio.'
  if (props.invoice.status === 'draft') return 'Si invia quando la fattura è emessa.'
  if (props.invoice.status === 'cancelled') return 'Una fattura annullata non si invia.'
  return 'Il PDF parte allegato a un’email per il paziente.'
})

/** A resend that failed after a successful send is worth a line of its own. */
const laterFailure = computed(() => {
  const { lastSent, last } = state.value
  return lastSent && last && last.status === 'failed' ? last : null
})
</script>

<template>
  <AppCard>
    <p class="label-quiet">Email al paziente</p>

    <div class="mt-3 flex items-start gap-3.5">
      <MailMark :kind="state.kind" :size="38" />
      <div class="min-w-0">
        <p class="text-md font-medium text-text">{{ describeMailMark(state.kind) }}</p>
        <p class="mt-0.5 break-words text-sm" :class="state.kind === 'failed' ? 'text-danger' : 'text-text-muted'">{{ sentence }}</p>
      </div>
    </div>

    <p
      v-if="laterFailure"
      class="mt-3 rounded-control bg-warn-soft px-3 py-2 text-xs leading-relaxed text-text-muted ring-1 ring-inset ring-warn-line"
      role="status"
    >
      Ultimo tentativo non riuscito ({{ formatUtcTimestamp(laterFailure.sent_at) }}): {{ laterFailure.error }}
    </p>

    <template v-if="sendable">
      <p v-if="email.loaded && !email.ready" class="mt-3 text-xs leading-relaxed text-text-muted">
        Per inviarla serve la casella email.
        <RouterLink to="/settings/email" class="rounded-sm font-medium text-accent hover:underline focus-ring">Configurala</RouterLink>
      </p>
      <AppButton
        class="mt-4"
        block
        :variant="state.kind === 'sent' ? 'secondary' : 'primary'"
        :icon="Send"
        :disabled="!email.ready"
        @click="dialogOpen = true"
      >
        {{ state.kind === 'sent' ? 'Invia di nuovo' : 'Invia al paziente' }}
      </AppButton>
    </template>

    <div v-if="history.length > 0" class="mt-4 border-t border-border pt-3">
      <button
        type="button"
        class="flex w-full items-center justify-between rounded-sm text-xs text-text-subtle hover:text-text focus-ring"
        :aria-expanded="historyOpen"
        @click="historyOpen = !historyOpen"
      >
        {{ history.length === 1 ? '1 invio registrato' : `${history.length} invii registrati` }}
        <ChevronDown :size="14" :stroke-width="1.8" class="transition-transform duration-200" :class="historyOpen ? 'rotate-180' : ''" aria-hidden="true" />
      </button>
      <Transition name="pane">
        <ol v-if="historyOpen" class="mt-2.5 space-y-2.5">
          <li v-for="record in history" :key="record.id" class="flex items-start gap-2.5 text-xs">
            <span class="mt-1 size-1.5 shrink-0 rounded-full" :class="record.status === 'sent' ? 'bg-accent' : 'bg-danger'" aria-hidden="true" />
            <div class="min-w-0 flex-1">
              <p class="flex items-baseline justify-between gap-2">
                <span class="truncate text-text">{{ record.recipient }}</span>
                <span class="tabular shrink-0 text-text-subtle">{{ formatUtcTimestamp(record.sent_at) }}</span>
              </p>
              <p v-if="record.status === 'failed'" class="mt-0.5 break-words text-danger">{{ record.error }}</p>
              <p v-else class="mt-0.5 truncate text-text-subtle">{{ record.subject }}</p>
            </div>
          </li>
        </ol>
      </Transition>
    </div>

    <SendInvoiceEmailDialog :open="dialogOpen" :invoice="invoice" @close="dialogOpen = false" />
  </AppCard>
</template>
