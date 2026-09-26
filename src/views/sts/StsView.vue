<script setup lang="ts">
/**
 * The Sistema Tessera Sanitaria page: what is still to send for the year,
 * every transmission with its outcome, and what the Sistema TS actually
 * holds month by month. Sending runs in the background every minute; the
 * header button sends now.
 */
import { computed, onMounted, ref, watch } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ArrowRight, CalendarClock, CircleCheck, ExternalLink, Hourglass, IdCard, KeyRound, Send, Settings2, UploadCloud } from 'lucide-vue-next'
import { listClients, listInvoices } from '@/api'
import { useStsStore } from '@/stores/sts'
import { useToastStore } from '@/stores/toast'
import type { Client, Invoice, TsDispatchSummary, TsSubmission } from '@/types'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import StatTile from '@/components/ui/StatTile.vue'
import type { SegmentOption } from '@/components/ui/types'
import StsHistoryTable from '@/components/sts/StsHistoryTable.vue'
import StsPendingTable from '@/components/sts/StsPendingTable.vue'
import StsRemoteReport from '@/components/sts/StsRemoteReport.vue'
import { formatCurrency, formatDateLong, todayIso } from '@/utils/format'
import { plural } from '@/utils/labels'
import { datePhrase, describeDispatch, stsDeadlinePhase, stsDeadlines } from '@/utils/sts'

type Tab = 'pending' | 'history' | 'remote'

const sts = useStsStore()
const toast = useToastStore()

const currentYear = new Date().getFullYear()
const YEAR_OPTIONS = [currentYear, currentYear - 1, currentYear - 2]

const year = ref(currentYear)
const tab = ref<Tab>('pending')
const paidInvoices = ref<Invoice[]>([])
const clients = ref(new Map<number, Client>())
const loading = ref(true)
const sending = ref(false)
const busyId = ref<number | null>(null)
const toCancel = ref<TsSubmission | null>(null)

async function loadYear(): Promise<void> {
  paidInvoices.value = await listInvoices({ year: year.value, status: 'paid' })
}

onMounted(async () => {
  try {
    const [, clientList] = await Promise.all([sts.load(), listClients(), loadYear()])
    clients.value = new Map(clientList.map((client) => [client.id, client]))
  } catch (error) {
    toast.notifyError(error, 'Sistema TS non leggibile')
  } finally {
    loading.value = false
  }
})

watch(year, () => { void loadYear() })

const pending = computed(() =>
  paidInvoices.value.filter((invoice) => ['none', 'rejected', 'cancelled'].includes(sts.stateOf(invoice.id).kind)),
)
const registered = computed(() => paidInvoices.value.filter((invoice) => sts.stateOf(invoice.id).kind === 'accepted'))
const pendingTotal = computed(() => pending.value.reduce((sum, invoice) => sum + invoice.total_gross, 0))
const registeredTotal = computed(() => registered.value.reduce((sum, invoice) => sum + invoice.total_gross, 0))

const yearSubmissions = computed(() => sts.submissions.filter((s) => s.invoice_year === year.value))

const liveIds = computed(() => {
  const ids = new Set<number>()
  for (const invoiceId of new Set(sts.submissions.map((s) => s.invoice_id))) {
    const live = sts.stateOf(invoiceId).live
    if (live) ids.add(live.id)
  }
  return ids
})

const deadline = computed(() => {
  const dates = stsDeadlines(year.value)
  const phase = stsDeadlinePhase(`${year.value}-12-31`, todayIso())
  const tone = phase === 'open' ? (pending.value.length > 0 ? 'accent' : 'safe') : phase === 'correction' ? 'warn' : 'danger'
  return { ...dates, phase, tone } as const
})

const tabs = computed<SegmentOption<Tab>[]>(() => [
  { value: 'pending', label: 'Da trasmettere', count: pending.value.length },
  { value: 'history', label: 'Trasmissioni', count: yearSubmissions.value.length },
  { value: 'remote', label: 'Sul Sistema TS' },
])

function announce(summary: TsDispatchSummary): void {
  const line = describeDispatch(summary)
  if (line.tone === 'success') toast.notify(line.message)
  else if (line.tone === 'error') toast.notifyError(line.message)
  else toast.notifyInfo(line.message)
}

async function sendNow(): Promise<void> {
  sending.value = true
  try {
    announce(await sts.dispatch())
  } catch (error) {
    toast.notifyError(error, 'Invio non riuscito')
  } finally {
    sending.value = false
  }
}

async function send(invoiceIds: number[]): Promise<void> {
  sending.value = true
  try {
    const { summary, refused } = await sts.send(invoiceIds)
    for (const { invoiceId, reason } of refused) {
      const invoice = paidInvoices.value.find((i) => i.id === invoiceId)
      toast.notifyError(reason, invoice ? `Fattura N. ${invoice.invoice_number}` : 'Fattura')
    }
    announce(summary)
  } catch (error) {
    toast.notifyError(error, 'Invio non riuscito')
  } finally {
    sending.value = false
  }
}

async function withRow(submission: TsSubmission, action: () => Promise<void>): Promise<void> {
  busyId.value = submission.id
  try {
    await action()
  } catch (error) {
    toast.notifyError(error, 'Sistema TS')
  } finally {
    busyId.value = null
  }
}

function withdraw(submission: TsSubmission): void {
  void withRow(submission, async () => {
    await sts.withdraw(submission.id)
    toast.notify(`Fattura N. ${submission.invoice_number} tolta dalla coda`)
  })
}

function resend(submission: TsSubmission): void {
  void withRow(submission, () => send([submission.invoice_id]))
}

async function openPortal(): Promise<void> {
  try {
    await openUrl('https://sistemats1.sanita.finanze.it/portale/')
  } catch (error) {
    toast.notifyError(error, 'Non riesco ad aprire il browser')
  }
}

async function confirmCancel(): Promise<void> {
  const submission = toCancel.value
  toCancel.value = null
  if (submission) await withRow(submission, async () => announce(await sts.cancel(submission.id)))
}
</script>

<template>
  <div>
    <PageHeader title="Sistema TS" subtitle="Le spese sanitarie dei pazienti, per la dichiarazione precompilata." :icon="IdCard">
      <AppBadge v-if="sts.environment === 'test'" tone="warn" mono>AMBIENTE DI TEST</AppBadge>
      <AppButton :icon="Settings2" to="/settings">Credenziali</AppButton>
      <AppButton variant="primary" :icon="Send" :loading="sending || sts.dispatching" @click="sendNow">Invia ora</AppButton>
    </PageHeader>

    <div class="page pt-6 pb-28">
      <!-- Until both secrets are stored nothing can leave: say so first, and where to find them. -->
      <section
        v-if="sts.loaded && !sts.connected"
        class="connect-banner settle mb-5 flex flex-wrap items-center gap-4 rounded-card border border-accent-line px-5 py-4"
        aria-labelledby="connect-title"
      >
        <span class="icon-chip" style="--chip: var(--accent)" aria-hidden="true"><KeyRound :size="16" :stroke-width="1.8" /></span>
        <div class="min-w-0 flex-1">
          <h2 id="connect-title" class="text-md font-semibold text-text">Collega il Sistema TS</h2>
          <p class="mt-0.5 text-sm text-text-muted">
            Servono la password e il PINCODE del Sistema TS: li trovi su sistemats.it, in <span class="font-medium text-text">Profilo utente → Stampa credenziali</span>.
            Finché mancano, le fatture restano in coda.
          </p>
        </div>
        <div class="flex shrink-0 gap-2">
          <AppButton variant="ghost" :icon="ExternalLink" @click="openPortal">Apri sistemats.it</AppButton>
          <AppButton variant="primary" :icon-right="ArrowRight" :to="{ path: '/settings', query: { focus: 'sts' } }">Inserisci le credenziali</AppButton>
        </div>
      </section>

      <div class="settle mb-5 grid grid-cols-2 gap-4 xl:grid-cols-4">
        <StatTile label="Da trasmettere" :value="String(pending.length)" :icon="UploadCloud" :tone="pending.length > 0 ? 'accent' : 'safe'">
          <template #hint>{{ pending.length > 0 ? `${formatCurrency(pendingTotal)} di spese ${year}` : `Tutto il ${year} è sul Sistema TS` }}</template>
        </StatTile>
        <StatTile label="In coda" :value="String(sts.counts.queued)" :icon="Hourglass" tone="neutral" hint="Invio automatico ogni minuto" />
        <StatTile label="Registrate" :value="String(registered.length)" :icon="CircleCheck" tone="safe">
          <template #hint>
            <span v-if="sts.counts.rejected > 0" class="font-medium text-danger">{{ plural(sts.counts.rejected, 'scartata da rivedere', 'scartate da rivedere') }}</span>
            <template v-else>{{ formatCurrency(registeredTotal) }} nel {{ year }}</template>
          </template>
        </StatTile>
        <StatTile :label="`Scadenza spese ${year}`" :value="formatDateLong(deadline.transmission)" :icon="CalendarClock" :tone="deadline.tone">
          <template #hint>
            <template v-if="deadline.phase === 'late'">Termini chiusi: invii e correzioni ora sono tardivi</template>
            <template v-else>Correzioni gratuite {{ datePhrase('fino', deadline.correction) }}</template>
          </template>
        </StatTile>
      </div>

      <div class="settle mb-3 flex flex-wrap items-center gap-3" style="--settle: 1">
        <select v-model.number="year" class="field field-sm w-auto pr-8" aria-label="Anno delle spese">
          <option v-for="option in YEAR_OPTIONS" :key="option" :value="option">Spese {{ option }}</option>
        </select>
        <SegmentedControl v-model="tab" :options="tabs" label="Sezione" size="sm" />
      </div>

      <AppCard :padded="false" class="settle overflow-hidden" style="--settle: 2">
        <SkeletonRows v-if="loading" variant="table" :count="6" label="Caricamento del Sistema TS" />
        <Transition v-else name="pane" mode="out-in">
          <div v-if="tab === 'pending'" key="pending">
            <StsPendingTable :invoices="pending" :clients="clients" :sending="sending" @send="send" />
          </div>
          <div v-else-if="tab === 'history'" key="history">
            <StsHistoryTable :submissions="yearSubmissions" :live-ids="liveIds" :busy-id="busyId" @withdraw="withdraw" @cancel="toCancel = $event" @resend="resend" />
          </div>
          <div v-else key="remote">
            <StsRemoteReport />
          </div>
        </Transition>
      </AppCard>
    </div>

    <ConfirmDialog
      :open="toCancel !== null"
      title="Annullare l’invio al Sistema TS?"
      message="Il documento viene cancellato dal Sistema TS e non comparirà nella precompilata del paziente."
      :blast-radius="toCancel ? `La fattura N. ${toCancel.invoice_number}/${toCancel.invoice_year} di ${toCancel.client_name} sparisce dal Sistema TS${toCancel.environment === 'test' ? ' (ambiente di test)' : ''}.` : undefined"
      confirm-label="Annulla invio"
      @confirm="confirmCancel"
      @cancel="toCancel = null"
    />
  </div>
</template>

<style scoped>
/* The one thing to do before anything else: an accent wash from the icon's corner, not an alarm. */
.connect-banner {
  background:
    radial-gradient(90% 140% at 0% 0%, var(--color-accent-soft), transparent 60%),
    var(--surface-raised);
}
</style>
