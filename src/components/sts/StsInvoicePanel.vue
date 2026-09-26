<script setup lang="ts">
/**
 * One invoice and the Sistema TS: where it stands, how long there is to
 * send or correct it, the next action, and a live check of what the
 * Sistema TS actually holds. The history lists every transmission.
 */
import { computed, onMounted, ref } from 'vue'
import { Ban, ChevronDown, RefreshCw, ScanSearch, Send, Undo2 } from 'lucide-vue-next'
import { queryTsInvoice } from '@/api'
import { useStsStore } from '@/stores/sts'
import { useToastStore } from '@/stores/toast'
import type { Invoice, TsDispatchSummary, TsQueryResult } from '@/types'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import TsMark from '@/components/ui/TsMark.vue'
import { formatCurrency, formatDateLong, todayIso } from '@/utils/format'
import {
  TS_ENVIRONMENT_LABEL,
  TS_OPERATION_LABEL,
  TS_SUBMISSION_STATUS,
  datePhrase,
  describeDispatch,
  describeSendKind,
  describeTsMark,
  formatStsTimestamp,
  stsActions,
  stsDeadlinePhase,
  stsDeadlines,
} from '@/utils/sts'

const props = defineProps<{ invoice: Invoice }>()

const sts = useStsStore()
const toast = useToastStore()

const busy = ref<'send' | 'replace' | 'cancel' | 'withdraw' | 'query' | null>(null)
const cancelOpen = ref(false)
const historyOpen = ref(false)
const remote = ref<TsQueryResult | null>(null)

onMounted(async () => {
  if (sts.loaded) return
  try {
    await sts.load()
  } catch (error) {
    toast.notifyError(error, 'Sistema TS non leggibile')
  }
})

const state = computed(() => sts.stateOf(props.invoice.id))
const actions = computed(() => stsActions(state.value, props.invoice.status))
const history = computed(() =>
  sts.submissions.filter((s) => s.invoice_id === props.invoice.id).sort((a, b) => b.id - a.id),
)
const isTest = computed(() => sts.environment === 'test')

const sentence = computed(() => {
  const current = state.value.current
  switch (state.value.kind) {
    case 'none':
      return props.invoice.status === 'paid'
        ? 'Spesa sanitaria da trasmettere.'
        : 'Si trasmette quando la fattura risulta pagata.'
    case 'queued':
      return current?.last_error
        ? `Nuovo tentativo: ${formatStsTimestamp(current.next_attempt_at)}.`
        : `${TS_OPERATION_LABEL[current?.operation ?? 'invio']} in attesa di invio.`
    case 'sending':
      return 'Invio in corso…'
    case 'accepted':
      return current?.sent_at ? `Registrata il ${formatStsTimestamp(current.sent_at)}.` : 'Registrata.'
    case 'rejected':
      return 'Il Sistema TS non l’ha accettata: correggi i dati e ritrasmetti.'
    case 'cancelled':
      return 'Il documento è stato cancellato dal Sistema TS.'
  }
  return ''
})

/** The deadline that matters for this payment year, and how close it is. */
const deadline = computed(() => {
  const paid = props.invoice.paid_date
  if (props.invoice.status !== 'paid' || !paid) return null
  const dates = stsDeadlines(Number(paid.slice(0, 4)))
  const phase = stsDeadlinePhase(paid, todayIso())
  const pending = state.value.kind !== 'accepted'
  if (phase === 'open') {
    return pending
      ? { tone: 'accent' as const, text: `Da inviare ${datePhrase('entro', dates.transmission)}` }
      : { tone: 'neutral' as const, text: `Correzioni senza sanzioni ${datePhrase('fino', dates.correction)}` }
  }
  if (phase === 'correction') {
    return { tone: 'warn' as const, text: `Ultimi giorni senza sanzioni: ${datePhrase('fino', dates.correction)}` }
  }
  return pending
    ? { tone: 'danger' as const, text: `Termine scaduto ${datePhrase('dal', dates.transmission)}: sarebbe un invio tardivo` }
    : null
})

async function run(kind: NonNullable<typeof busy.value>, action: () => Promise<TsDispatchSummary>): Promise<void> {
  busy.value = kind
  remote.value = null
  try {
    announce(await action())
  } catch (error) {
    toast.notifyError(error, 'Sistema TS')
  } finally {
    busy.value = null
  }
}

function announce(summary: TsDispatchSummary): void {
  const line = describeDispatch(summary)
  if (line.tone === 'success') toast.notify(line.message)
  else if (line.tone === 'error') toast.notifyError(line.message)
  else toast.notifyInfo(line.message)
}

async function send(): Promise<void> {
  await run('send', async () => {
    const { summary, refused } = await sts.send([props.invoice.id])
    if (refused.length > 0) throw refused[0]!.reason
    return summary
  })
}

async function replace(): Promise<void> {
  const live = state.value.live
  if (live) await run('replace', () => sts.replace(live.id))
}

async function confirmCancel(): Promise<void> {
  const live = state.value.live
  cancelOpen.value = false
  if (live) await run('cancel', () => sts.cancel(live.id))
}

async function withdraw(): Promise<void> {
  const current = state.value.current
  if (!current) return
  busy.value = 'withdraw'
  remote.value = null
  try {
    await sts.withdraw(current.id)
    toast.notify('Tolta dalla coda del Sistema TS')
  } catch (error) {
    toast.notifyError(error, 'Non ritirata')
  } finally {
    busy.value = null
  }
}

async function verify(): Promise<void> {
  busy.value = 'query'
  try {
    remote.value = await queryTsInvoice(props.invoice.id)
  } catch (error) {
    toast.notifyError(error, 'Verifica non riuscita')
  } finally {
    busy.value = null
  }
}

const remoteTotal = computed(() => {
  if (remote.value?.kind !== 'found') return 0
  return remote.value.document.totals.reduce((sum, total) => sum + total.amount, 0)
})
</script>

<template>
  <AppCard>
    <div class="flex items-center justify-between gap-2">
      <p class="label-quiet">Sistema Tessera Sanitaria</p>
      <AppBadge v-if="isTest" tone="warn" mono>TEST</AppBadge>
    </div>

    <div class="mt-3 flex items-start gap-3.5">
      <TsMark :kind="state.kind" :size="38" />
      <div class="min-w-0">
        <p class="text-md font-medium text-text">{{ describeTsMark(state.kind) }}</p>
        <p class="mt-0.5 text-sm text-text-muted">{{ sentence }}</p>
      </div>
    </div>

    <dl v-if="state.current?.protocol" class="mt-3 flex items-center justify-between rounded-control bg-surface-sunken px-3 py-2 text-xs">
      <dt class="text-text-subtle">Protocollo</dt>
      <dd class="font-mono tabular text-text">{{ state.current.protocol }}</dd>
    </dl>

    <p
      v-if="state.kind === 'rejected' && state.current?.outcome_message"
      class="mt-3 rounded-control bg-danger-soft px-3 py-2 text-xs leading-relaxed text-danger ring-1 ring-inset ring-danger-line"
      role="status"
    >
      {{ state.current.outcome_message }}
    </p>
    <p
      v-else-if="state.kind === 'queued' && state.current?.last_error"
      class="mt-3 rounded-control bg-warn-soft px-3 py-2 text-xs leading-relaxed text-text-muted ring-1 ring-inset ring-warn-line"
      role="status"
    >
      {{ state.current.last_error }}
    </p>
    <p
      v-else-if="state.failedFollowUp"
      class="mt-3 rounded-control bg-warn-soft px-3 py-2 text-xs leading-relaxed text-text-muted ring-1 ring-inset ring-warn-line"
      role="status"
    >
      {{ TS_OPERATION_LABEL[state.failedFollowUp.operation] }} scartata: {{ state.failedFollowUp.outcome_message ?? state.failedFollowUp.outcome_code }}. Sul Sistema TS restano i dati precedenti.
    </p>
    <p v-else-if="state.kind === 'accepted' && state.current?.outcome_message" class="mt-3 text-xs text-text-subtle">
      {{ state.current.outcome_message }}
    </p>

    <p v-if="deadline" class="mt-3 flex items-center gap-2 text-xs" :class="{
      'text-accent': deadline.tone === 'accent',
      'text-text-subtle': deadline.tone === 'neutral',
      'text-warn': deadline.tone === 'warn',
      'text-danger': deadline.tone === 'danger',
    }">
      <span class="size-1.5 shrink-0 rounded-full bg-current" aria-hidden="true" />
      {{ deadline.text }}
    </p>

    <div class="mt-4 space-y-2">
      <AppButton v-if="actions.send" variant="primary" block :icon="Send" :loading="busy === 'send'" @click="send">
        {{ state.kind === 'none' ? 'Trasmetti al Sistema TS' : 'Trasmetti di nuovo' }}
      </AppButton>
      <AppButton v-if="actions.replace" block :icon="RefreshCw" :loading="busy === 'replace'" @click="replace">
        Invia i dati aggiornati
      </AppButton>
      <AppButton v-if="actions.withdraw" block :icon="Undo2" :loading="busy === 'withdraw'" @click="withdraw">
        Togli dalla coda
      </AppButton>
      <div class="flex gap-2">
        <AppButton variant="ghost" size="sm" class="flex-1" :icon="ScanSearch" :loading="busy === 'query'" @click="verify">
          Verifica online
        </AppButton>
        <AppButton v-if="actions.cancel" variant="danger-quiet" size="sm" class="flex-1" :icon="Ban" :loading="busy === 'cancel'" @click="cancelOpen = true">
          Annulla invio
        </AppButton>
      </div>
    </div>

    <Transition name="pane">
      <div v-if="remote" class="mt-4 rounded-control border border-border bg-surface px-3 py-3 text-xs" aria-live="polite">
        <p class="mb-2 font-medium text-text">Sul Sistema TS ({{ TS_ENVIRONMENT_LABEL[sts.environment].toLowerCase() }})</p>
        <template v-if="remote.kind === 'found'">
          <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
            <dt class="text-text-subtle">Stato</dt>
            <dd class="text-right" :class="remote.document.cancelled ? 'text-text-muted' : 'text-safe'">
              {{ remote.document.cancelled ? 'Cancellato' : describeSendKind(remote.document.send_kind) }}
            </dd>
            <dt class="text-text-subtle">Importo</dt>
            <dd class="tabular text-right text-text">{{ formatCurrency(remoteTotal) }}</dd>
            <dt class="text-text-subtle">Pagamento</dt>
            <dd class="text-right text-text">{{ remote.document.payment_date ? formatDateLong(remote.document.payment_date) : '—' }}</dd>
            <dt class="text-text-subtle">Inviato</dt>
            <dd class="text-right text-text">{{ remote.document.sent_date ? formatDateLong(remote.document.sent_date) : '—' }}</dd>
          </dl>
          <p v-if="!remote.document.cancelled && Math.abs(remoteTotal - invoice.total_gross) > 0.009" class="mt-2 text-warn">
            L’importo registrato è diverso da quello della fattura ({{ formatCurrency(invoice.total_gross) }}): invia i dati aggiornati.
          </p>
        </template>
        <p v-else-if="remote.kind === 'not_found'" class="text-text-muted">Nessun documento con questo numero e data.</p>
        <p v-else class="text-danger">{{ remote.messages.map((m) => `${m.code} ${m.description}`).join(' · ') }}</p>
      </div>
    </Transition>

    <div v-if="history.length > 0" class="mt-4 border-t border-border pt-3">
      <button
        type="button"
        class="flex w-full items-center justify-between rounded-control text-xs text-text-subtle hover:text-text focus-ring"
        :aria-expanded="historyOpen"
        @click="historyOpen = !historyOpen"
      >
        Cronologia ({{ history.length }})
        <ChevronDown :size="14" class="transition-transform" :class="historyOpen ? 'rotate-180' : ''" aria-hidden="true" />
      </button>
      <ol v-if="historyOpen" class="mt-2 space-y-2">
        <li v-for="item in history" :key="item.id" class="flex items-center gap-2 text-xs">
          <AppBadge :tone="TS_SUBMISSION_STATUS[item.status].tone" dot>{{ TS_SUBMISSION_STATUS[item.status].label }}</AppBadge>
          <span class="flex-1 truncate text-text-muted">{{ TS_OPERATION_LABEL[item.operation] }}<span v-if="item.environment === 'test'" class="text-text-subtle"> · test</span></span>
          <span class="tabular text-text-subtle">{{ formatStsTimestamp(item.resolved_at ?? item.created_at) }}</span>
        </li>
      </ol>
    </div>

    <ConfirmDialog
      :open="cancelOpen"
      title="Annullare l’invio al Sistema TS?"
      message="Il documento viene cancellato dal Sistema TS e non comparirà nella precompilata del paziente."
      :blast-radius="`La spesa di ${formatCurrency(invoice.total_gross)} della fattura N. ${invoice.invoice_number}/${invoice.year} di ${invoice.client_name} sparisce dal Sistema TS${isTest ? ' (ambiente di test)' : ''}.`"
      confirm-label="Annulla invio"
      :loading="busy === 'cancel'"
      @confirm="confirmCancel"
      @cancel="cancelOpen = false"
    />
  </AppCard>
</template>
