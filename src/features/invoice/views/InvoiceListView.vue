<script setup lang="ts">
/**
 * All invoices of a year. The year is loaded once; status and search filter
 * locally, so the status tabs can show their counts and typing never waits
 * on the database. Filters live in the URL, so the dashboard can link here
 * already filtered ("3 fatture scadute" → ?status=overdue).
 */
import { computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { CalendarRange, CircleCheck, FileText, Hourglass, Pencil, Plus, Search, Send, Sigma, Trash2, X } from 'lucide-vue-next'
import { useInvoicesStore } from '@/features/invoice/store/invoices'
import { useToastStore } from '@/common/store/toast'
import { useEmailStore } from '@/features/email/store/email'
import { useStsStore } from '@/features/sts/store/sts'
import type { InvoiceStatus } from '@/features/invoice/model/invoice'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import InvoiceSeal from '@/components/ui/InvoiceSeal.vue'
import TsMark from '@/components/ui/TsMark.vue'
import MailMark from '@/components/ui/MailMark.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import StatTile from '@/components/ui/StatTile.vue'
import SparkLine from '@/components/ui/SparkLine.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { useInvoiceListFilters } from '@/features/invoice/composables/useInvoiceListFilters'
import { useInvoiceSelection } from '@/features/invoice/composables/useInvoiceSelection'
import { useBulkInvoiceStatus } from '@/features/invoice/composables/useBulkInvoiceStatus'
import { useBulkInvoiceEmail } from '@/features/email/composables/useBulkInvoiceEmail'
import { useInvoiceDeletion } from '@/features/invoice/composables/useInvoiceDeletion'
import { formatCurrency, formatDate } from '@/common/lib/format'
import { PAYMENT_METHOD_LABEL, plural } from '@/common/lib/labels'
import {
  BULK_STATUS_OPTIONS as BULK_OPTIONS,
  invoicesInStatus,
  monthlyTotals as monthlyTotalsOf,
  paidShare as paidShareOf,
  searchInvoices,
  statusDistribution,
  statusFilterOptions,
  summarise,
  yearOptions,
} from '@/features/invoice/lib/invoiceList'

const router = useRouter()
const invoicesStore = useInvoicesStore()
const sts = useStsStore()
const email = useEmailStore()
const toast = useToastStore()
const currentYear = new Date().getFullYear()
const YEAR_OPTIONS = yearOptions(currentYear)

const { year, status, search, clearFilters } = useInvoiceListFilters(currentYear)
const searched = computed(() => searchInvoices(invoicesStore.invoices, search.value))
const statusOptions = computed(() => statusFilterOptions(searched.value))
const visible = computed(() => invoicesInStatus(searched.value, status.value))
const summary = computed(() => summarise(visible.value))
const paidShare = computed(() => paidShareOf(summary.value))
/** Only a single year has months to show. */
const monthlyTotals = computed(() => monthlyTotalsOf(visible.value))
const lastMonthIndex = computed(() => (year.value === currentYear ? new Date().getMonth() : 11))
const distribution = computed(() => statusDistribution(visible.value))

/** How the listed invoices split across statuses, drawn as one segmented bar. */
const STATUS_FILL: Record<InvoiceStatus, string> = {
  draft: 'bg-text-subtle/40',
  issued: 'bg-warn',
  paid: 'bg-safe',
  overdue: 'bg-danger',
  cancelled: 'bg-border-strong',
}

const selection = useInvoiceSelection(() => visible.value, () => invoicesStore.invoices)
const { selected, allVisibleSelected, someVisibleSelected, toggleAll, toggle, clearSelection } = selection
const { bulkTarget, bulkConfirmOpen, bulkRunning, bulkBlastRadius, runBulk } = useBulkInvoiceStatus(selection)
const { bulkEmailOpen, bulkEmailProgress, emailable, bulkEmailBlastRadius, bulkEmailMessage, runBulkEmail } = useBulkInvoiceEmail(selection)
const { toDelete, deleting, confirmDelete } = useInvoiceDeletion()

async function load(): Promise<void> {
  clearSelection()
  await invoicesStore.fetchInvoices({ year: year.value === 0 ? undefined : year.value })
  if (invoicesStore.error) toast.notifyError(invoicesStore.error, 'Caricamento non riuscito')
}

watch(year, load)
onMounted(() => {
  void load()
  // The Sistema TS and email marks are a courtesy: without them the list still works.
  sts.load().catch(() => undefined)
  email.load().catch(() => undefined)
})
</script>

<template>
  <div>
    <PageHeader title="Fatture" :icon="FileText">
      <AppButton :icon="CalendarRange" to="/invoices/monthly">Fatturazione mensile</AppButton>
      <AppButton variant="primary" :icon="Plus" to="/invoices/new">Nuova fattura</AppButton>
    </PageHeader>

    <div class="page pt-6 pb-28">
      <!-- Summary of what the filters show: four instruments, each with its own shape. -->
      <div class="settle mb-5 grid grid-cols-2 gap-4 xl:grid-cols-4">
        <StatTile label="Fatture" :value="String(summary.count)" :icon="FileText" tone="accent">
          <template #hint>{{ year === 0 ? 'Tutti gli anni' : `Nel ${year}` }}</template>
          <div class="flex h-1.5 gap-0.5 overflow-hidden rounded-full bg-surface-sunken" role="img" :aria-label="distribution.map((segment) => `${segment.label} ${segment.count}`).join(', ')">
            <span
              v-for="segment in distribution"
              :key="segment.value"
              class="h-full first:rounded-l-full last:rounded-r-full transition-[width] duration-500 ease-out-expo"
              :class="STATUS_FILL[segment.value]"
              :style="{ width: `${segment.share}%` }"
              :title="`${segment.label}: ${segment.count}`"
            />
          </div>
        </StatTile>
        <StatTile label="Totale fatturato" :value="formatCurrency(summary.total)" :icon="Sigma" tone="accent" hint="Escluse le annullate">
          <SparkLine v-if="year !== 0" :values="monthlyTotals" :last-index="lastMonthIndex" :height="28" />
        </StatTile>
        <StatTile label="Incassato" :value="formatCurrency(summary.paid)" :icon="CircleCheck" tone="safe" :hint="`${paidShare}% del totale`">
          <div class="h-1.5 overflow-hidden rounded-full bg-safe-soft">
            <div class="h-full rounded-full bg-gradient-to-r from-safe/70 to-safe transition-[width] duration-700 ease-out-expo" :style="{ width: `${paidShare}%` }" />
          </div>
        </StatTile>
        <StatTile label="In attesa" :value="formatCurrency(summary.pending)" :icon="Hourglass" :tone="summary.overdue > 0 ? 'danger' : 'warn'">
          <template #hint>
            <span v-if="summary.overdue > 0" class="font-medium text-danger">{{ plural(summary.overdue, 'fattura scaduta', 'fatture scadute') }}</span>
            <template v-else>Nessuna scadenza superata</template>
          </template>
        </StatTile>
      </div>

      <!-- Filters: one row, above the list. -->
      <div class="settle mb-3 flex flex-wrap items-center gap-3" style="--settle: 1">
        <select v-model.number="year" class="field field-sm w-auto pr-8" aria-label="Anno">
          <option v-for="option in YEAR_OPTIONS" :key="option.value" :value="option.value">{{ option.label }}</option>
        </select>
        <SegmentedControl v-model="status" :options="statusOptions" label="Stato" size="sm" />
        <div class="relative ml-auto w-full sm:w-64 2xl:w-80">
          <Search :size="15" class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-text-subtle" aria-hidden="true" />
          <input v-model="search" type="search" class="field field-sm pl-8" placeholder="Numero o paziente" aria-label="Cerca fatture" />
        </div>
      </div>

      <AppCard :padded="false" class="settle overflow-hidden" style="--settle: 2">
        <SkeletonRows v-if="invoicesStore.loading" variant="table" :count="8" label="Caricamento delle fatture" />

        <EmptyState
          v-else-if="visible.length === 0"
          :icon="FileText"
          :bordered="false"
          :title="invoicesStore.invoices.length === 0 ? 'Nessuna fattura in questo periodo' : 'Nessuna fattura corrisponde ai filtri'"
          :description="invoicesStore.invoices.length === 0 ? 'Crea la prima, oppure genera le fatture del mese dalle sedute svolte.' : undefined"
        >
          <AppButton v-if="invoicesStore.invoices.length > 0" @click="clearFilters">Azzera i filtri</AppButton>
          <AppButton v-else variant="primary" :icon="Plus" to="/invoices/new">Nuova fattura</AppButton>
        </EmptyState>

        <table v-else class="data-table text-base">
          <thead>
            <tr>
              <th class="w-12 pl-5 font-normal">
                <input
                  type="checkbox"
                  :checked="allVisibleSelected"
                  :indeterminate="someVisibleSelected"
                  aria-label="Seleziona tutte le fatture visibili"
                  @change="toggleAll"
                />
              </th>
              <th class="w-8 font-normal"><span class="sr-only">Stato</span></th>
              <th class="w-8 font-normal"><span class="sr-only">Sistema TS</span></th>
              <th class="w-8 font-normal"><span class="sr-only">Email al paziente</span></th>
              <th class="w-24 font-medium">Numero</th>
              <th class="font-medium">Paziente</th>
              <th class="w-28 font-medium">Emessa</th>
              <th class="hidden w-28 font-medium xl:table-cell">Scadenza</th>
              <th class="hidden w-36 font-medium 2xl:table-cell">Pagamento</th>
              <th class="w-32 pr-6 text-right font-medium">Importo</th>
              <th class="w-28 font-medium">Stato</th>
              <th class="w-24 pr-4"><span class="sr-only">Azioni</span></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="invoice in visible"
              :key="invoice.id"
              class="group h-row cursor-pointer"
              data-row
              :data-selected="selected.has(invoice.id)"
              @click="router.push(`/invoices/${invoice.id}`)"
            >
              <td class="pl-5" @click.stop>
                <input
                  type="checkbox"
                  :checked="selected.has(invoice.id)"
                  :aria-label="`Seleziona la fattura N. ${invoice.invoice_number}`"
                  @change="toggle(invoice.id)"
                />
              </td>
              <td><InvoiceSeal :status="invoice.status" :issue-date="invoice.issue_date" :due-date="invoice.due_date" :size="18" /></td>
              <td><TsMark v-if="invoice.status === 'paid' || sts.stateOf(invoice.id).kind !== 'none'" :kind="sts.stateOf(invoice.id).kind" :size="18" /></td>
              <td>
                <MailMark
                  v-if="email.loaded && ((invoice.status !== 'draft' && invoice.status !== 'cancelled') || email.stateOf(invoice.id).kind !== 'none')"
                  :kind="email.stateOf(invoice.id).kind"
                  :size="18"
                />
              </td>
              <td class="tabular text-text-muted">N. {{ invoice.invoice_number }}<span v-if="year === 0" class="text-text-subtle">/{{ invoice.year }}</span></td>
              <td>
                <span class="flex min-w-0 items-center gap-2.5">
                  <PatientMonogram :name="invoice.client_name" size="sm" />
                  <span class="truncate text-text">{{ invoice.client_name }}</span>
                </span>
              </td>
              <td class="tabular text-sm text-text-muted">{{ formatDate(invoice.issue_date) }}</td>
              <td class="tabular hidden text-sm xl:table-cell" :class="invoice.status === 'overdue' ? 'font-medium text-danger' : 'text-text-subtle'">{{ invoice.due_date ? formatDate(invoice.due_date) : '—' }}</td>
              <td class="hidden text-sm text-text-muted 2xl:table-cell">{{ PAYMENT_METHOD_LABEL[invoice.payment_method] }}</td>
              <td class="tabular pr-6 text-right font-medium text-text">{{ formatCurrency(invoice.total_due) }}</td>
              <td><StatusBadge type="invoice" :status="invoice.status" /></td>
              <td class="pr-4" @click.stop>
                <div class="flex justify-end gap-0.5 opacity-60 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
                  <AppButton variant="ghost" size="sm" :icon="Pencil" :label="`Modifica la fattura N. ${invoice.invoice_number}`" :to="`/invoices/${invoice.id}/edit`" />
                  <AppButton variant="danger-quiet" size="sm" :icon="Trash2" :label="`Elimina la fattura N. ${invoice.invoice_number}`" @click="toDelete = invoice" />
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </AppCard>
    </div>

    <!-- Bulk bar: floats above the list while something is selected. -->
    <Teleport to="body">
      <Transition name="rise">
        <div
          v-if="selected.size > 0"
          class="fixed bottom-6 left-[calc(50%+var(--spacing-sidebar)/2)] z-(--z-overlay) flex -translate-x-1/2 items-center gap-3 rounded-card border border-border bg-surface-raised py-2 pl-4 pr-2 shadow-modal"
          role="region"
          aria-label="Azioni sulle fatture selezionate"
        >
          <span class="whitespace-nowrap text-base font-medium text-text">{{ plural(selected.size, 'selezionata', 'selezionate') }}</span>
          <span class="h-5 w-px bg-border" aria-hidden="true" />
          <span class="whitespace-nowrap text-sm text-text-muted">Segna come</span>
          <SegmentedControl v-model="bulkTarget" :options="BULK_OPTIONS" label="Nuovo stato" size="sm" />
          <AppButton variant="primary" size="sm" @click="bulkConfirmOpen = true">Applica</AppButton>
          <span class="h-5 w-px bg-border" aria-hidden="true" />
          <span :title="email.ready ? undefined : 'Configura prima la casella in Impostazioni → Email'">
            <AppButton size="sm" :icon="Send" :disabled="!email.ready || emailable.length === 0" @click="bulkEmailOpen = true">Invia via email</AppButton>
          </span>
          <AppButton variant="ghost" size="sm" :icon="X" label="Annulla selezione" @click="selected = new Set()" />
        </div>
      </Transition>
    </Teleport>

    <ConfirmDialog
      :open="bulkConfirmOpen"
      title="Cambiare lo stato?"
      :blast-radius="bulkBlastRadius"
      confirm-label="Conferma"
      tone="accent"
      :loading="bulkRunning"
      @confirm="runBulk"
      @cancel="bulkConfirmOpen = false"
    />
    <ConfirmDialog
      :open="bulkEmailOpen"
      title="Inviare le fatture ai pazienti?"
      :message="bulkEmailMessage"
      :blast-radius="bulkEmailBlastRadius"
      :confirm-label="`Invia ${emailable.length}`"
      tone="accent"
      :loading="bulkEmailProgress !== null"
      @confirm="runBulkEmail"
      @cancel="bulkEmailOpen = false"
    />
    <ConfirmDialog
      :open="toDelete !== null"
      title="Eliminare la fattura?"
      message="L'operazione non si può annullare."
      :blast-radius="toDelete ? `La fattura N. ${toDelete.invoice_number}/${toDelete.year} di ${toDelete.client_name}, ${formatCurrency(toDelete.total_due)}, verrà eliminata definitivamente.` : undefined"
      :loading="deleting"
      @confirm="confirmDelete"
      @cancel="toDelete = null"
    />
  </div>
</template>
