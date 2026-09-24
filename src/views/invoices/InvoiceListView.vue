<script setup lang="ts">
/**
 * All invoices of a year. The year is loaded once; status and search filter
 * locally, so the status tabs can show their counts and typing never waits
 * on the database. Filters live in the URL, so the dashboard can link here
 * already filtered ("3 fatture scadute" → ?status=overdue).
 */
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { CalendarRange, FileText, Pencil, Plus, Search, Trash2, X } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import { useToastStore } from '@/stores/toast'
import type { Invoice, InvoiceStatus } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import InvoiceSeal from '@/components/ui/InvoiceSeal.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import type { SegmentOption } from '@/components/ui/types'
import { formatCurrency, formatDate, todayIso } from '@/utils/format'
import { INVOICE_STATUS, INVOICE_STATUS_ORDER, isInvoiceStatus, plural } from '@/utils/labels'

type StatusFilter = InvoiceStatus | 'all'

const route = useRoute()
const router = useRouter()
const invoicesStore = useInvoicesStore()
const toast = useToastStore()
const currentYear = new Date().getFullYear()

function queryYear(): number | 0 {
  const raw = Number(route.query.year)
  return Number.isInteger(raw) && raw >= 0 ? raw : currentYear
}
function queryStatus(): StatusFilter {
  const raw = String(route.query.status ?? '')
  return isInvoiceStatus(raw) ? raw : 'all'
}

/** 0 means every year. */
const year = ref<number>(queryYear())
const status = ref<StatusFilter>(queryStatus())
const search = ref(String(route.query.q ?? ''))

watch([year, status], () => {
  void router.replace({
    query: {
      ...(year.value !== currentYear ? { year: year.value } : {}),
      ...(status.value !== 'all' ? { status: status.value } : {}),
    },
  })
})

const YEAR_OPTIONS: SegmentOption<number>[] = [
  { value: currentYear, label: String(currentYear) },
  { value: currentYear - 1, label: String(currentYear - 1) },
  { value: currentYear - 2, label: String(currentYear - 2) },
  { value: 0, label: 'Tutti gli anni' },
]

function normalise(text: string): string {
  return text.toLocaleLowerCase('it-IT').normalize('NFD').replace(/\p{Diacritic}/gu, '')
}

const searched = computed(() => {
  const needle = normalise(search.value.trim())
  if (needle === '') return invoicesStore.invoices
  return invoicesStore.invoices.filter((invoice) =>
    normalise(`${invoice.invoice_number} ${invoice.client_name}`).includes(needle),
  )
})

const statusOptions = computed<SegmentOption<StatusFilter>[]>(() => [
  { value: 'all', label: 'Tutte', count: searched.value.length },
  ...INVOICE_STATUS_ORDER.map((value) => ({
    value,
    label: INVOICE_STATUS[value].label,
    count: searched.value.filter((invoice) => invoice.status === value).length,
  })),
])

const visible = computed(() =>
  status.value === 'all' ? searched.value : searched.value.filter((invoice) => invoice.status === status.value),
)

const summary = computed(() => {
  const live = visible.value.filter((invoice) => invoice.status !== 'cancelled')
  const sum = (list: Invoice[]): number => list.reduce((total, invoice) => total + invoice.total_due, 0)
  return {
    count: visible.value.length,
    total: sum(live),
    paid: sum(live.filter((invoice) => invoice.status === 'paid')),
    pending: sum(live.filter((invoice) => invoice.status === 'issued' || invoice.status === 'overdue')),
  }
})

async function load(): Promise<void> {
  selected.value = new Set()
  await invoicesStore.fetchInvoices({ year: year.value === 0 ? undefined : year.value })
  if (invoicesStore.error) toast.notifyError(invoicesStore.error, 'Caricamento non riuscito')
}

watch(year, load)
onMounted(load)

// ─── Selection and bulk status ──────────────────────────────────────────────

const selected = ref<Set<number>>(new Set())
const bulkTarget = ref<InvoiceStatus>('paid')
const bulkConfirmOpen = ref(false)
const bulkRunning = ref(false)

const allVisibleSelected = computed(
  () => visible.value.length > 0 && visible.value.every((invoice) => selected.value.has(invoice.id)),
)
const someVisibleSelected = computed(
  () => !allVisibleSelected.value && visible.value.some((invoice) => selected.value.has(invoice.id)),
)
const selectedInvoices = computed(() => invoicesStore.invoices.filter((invoice) => selected.value.has(invoice.id)))

function toggleAll(): void {
  selected.value = allVisibleSelected.value ? new Set() : new Set(visible.value.map((invoice) => invoice.id))
}

function toggle(id: number): void {
  const next = new Set(selected.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selected.value = next
}

const BULK_OPTIONS: SegmentOption<InvoiceStatus>[] = (['issued', 'paid', 'overdue', 'cancelled'] as const).map((value) => ({
  value,
  label: INVOICE_STATUS[value].label,
  tone: INVOICE_STATUS[value].tone,
}))

const bulkBlastRadius = computed(() => {
  const count = selectedInvoices.value.length
  const amount = selectedInvoices.value.reduce((total, invoice) => total + invoice.total_due, 0)
  const target = INVOICE_STATUS[bulkTarget.value].label.toLocaleLowerCase('it-IT')
  const paidNote = bulkTarget.value === 'paid' ? ', con data di pagamento oggi' : ''
  return `${plural(count, 'fattura', 'fatture')} per ${formatCurrency(amount)} ${count === 1 ? 'passerà' : 'passeranno'} a “${target}”${paidNote}.`
})

async function runBulk(): Promise<void> {
  bulkRunning.value = true
  const ids = [...selected.value]
  try {
    await invoicesStore.bulkUpdateStatus(ids, bulkTarget.value, bulkTarget.value === 'paid' ? todayIso() : undefined)
    toast.notify(`${plural(ids.length, 'fattura aggiornata', 'fatture aggiornate')}`)
    selected.value = new Set()
    bulkConfirmOpen.value = false
  } catch (error) {
    toast.notifyError(error, 'Aggiornamento non riuscito')
  } finally {
    bulkRunning.value = false
  }
}

// ─── Delete ─────────────────────────────────────────────────────────────────

const toDelete = ref<Invoice | null>(null)
const deleting = ref(false)

async function confirmDelete(): Promise<void> {
  const invoice = toDelete.value
  if (invoice === null) return
  deleting.value = true
  try {
    await invoicesStore.removeInvoice(invoice.id)
    toast.notify(`Fattura N. ${invoice.invoice_number} eliminata`)
    toDelete.value = null
  } catch (error) {
    toast.notifyError(error, 'Eliminazione non riuscita')
  } finally {
    deleting.value = false
  }
}

function clearFilters(): void {
  search.value = ''
  status.value = 'all'
}
</script>

<template>
  <div>
    <PageHeader title="Fatture">
      <AppButton :icon="CalendarRange" to="/invoices/monthly">Fatturazione mensile</AppButton>
      <AppButton variant="primary" :icon="Plus" to="/invoices/new">Nuova fattura</AppButton>
    </PageHeader>

    <div class="mx-auto max-w-[72rem] px-8 pt-6 pb-28">
      <!-- Summary of what the filters show. -->
      <dl class="settle mb-5 grid grid-cols-4 divide-x divide-border rounded-card border border-border bg-surface-raised">
        <div class="px-5 py-4">
          <dt class="label-quiet">Fatture</dt>
          <dd class="mt-0.5 text-xl font-semibold text-text">{{ summary.count }}</dd>
        </div>
        <div class="px-5 py-4">
          <dt class="label-quiet">Totale</dt>
          <dd class="mt-0.5 text-xl font-semibold text-text">{{ formatCurrency(summary.total) }}</dd>
        </div>
        <div class="px-5 py-4">
          <dt class="label-quiet flex items-center gap-1.5"><span class="size-1.5 rounded-full bg-safe" aria-hidden="true" />Incassato</dt>
          <dd class="mt-0.5 text-xl font-semibold text-text">{{ formatCurrency(summary.paid) }}</dd>
        </div>
        <div class="px-5 py-4">
          <dt class="label-quiet flex items-center gap-1.5"><span class="size-1.5 rounded-full bg-warn" aria-hidden="true" />In attesa</dt>
          <dd class="mt-0.5 text-xl font-semibold text-text">{{ formatCurrency(summary.pending) }}</dd>
        </div>
      </dl>

      <!-- Filters: one row, above the list. -->
      <div class="settle mb-3 flex flex-wrap items-center gap-3" style="--settle: 1">
        <select v-model.number="year" class="field field-sm w-auto pr-8" aria-label="Anno">
          <option v-for="option in YEAR_OPTIONS" :key="option.value" :value="option.value">{{ option.label }}</option>
        </select>
        <SegmentedControl v-model="status" :options="statusOptions" label="Stato" size="sm" />
        <div class="relative ml-auto w-52">
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

        <table v-else class="w-full text-base">
          <thead class="border-b border-border bg-surface text-left text-xs text-text-subtle">
            <tr class="h-9">
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
              <th class="w-24 font-medium">Numero</th>
              <th class="font-medium">Paziente</th>
              <th class="w-28 font-medium">Emessa</th>
              <th class="w-32 pr-6 text-right font-medium">Importo</th>
              <th class="w-28 font-medium">Stato</th>
              <th class="w-24 pr-4"><span class="sr-only">Azioni</span></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="invoice in visible"
              :key="invoice.id"
              class="group h-row cursor-pointer border-b border-border last:border-b-0 transition-colors"
              :class="selected.has(invoice.id) ? 'bg-accent-soft' : 'hover:bg-surface-hover'"
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
              <td class="tabular text-text-muted">N. {{ invoice.invoice_number }}<span v-if="year === 0" class="text-text-subtle">/{{ invoice.year }}</span></td>
              <td>
                <span class="flex min-w-0 items-center gap-2.5">
                  <PatientMonogram :name="invoice.client_name" size="sm" />
                  <span class="truncate text-text">{{ invoice.client_name }}</span>
                </span>
              </td>
              <td class="tabular text-sm text-text-muted">{{ formatDate(invoice.issue_date) }}</td>
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
          class="fixed bottom-6 left-[calc(50%+7.5rem)] z-(--z-overlay) flex -translate-x-1/2 items-center gap-3 rounded-card border border-border bg-surface-raised py-2 pl-4 pr-2 shadow-modal"
          role="region"
          aria-label="Azioni sulle fatture selezionate"
        >
          <span class="whitespace-nowrap text-base font-medium text-text">{{ plural(selected.size, 'selezionata', 'selezionate') }}</span>
          <span class="h-5 w-px bg-border" aria-hidden="true" />
          <span class="whitespace-nowrap text-sm text-text-muted">Segna come</span>
          <SegmentedControl v-model="bulkTarget" :options="BULK_OPTIONS" label="Nuovo stato" size="sm" />
          <AppButton variant="primary" size="sm" @click="bulkConfirmOpen = true">Applica</AppButton>
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
