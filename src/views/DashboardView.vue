<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { ArrowRight, BarChart3, Calculator, CalendarRange, CircleCheck, FilePen, FileText, Gauge, Hourglass, LayoutGrid, ListChecks, Receipt, TriangleAlert } from 'lucide-vue-next'
import { getDashboard, listInvoices, previewMonthlyInvoices } from '@/api'
import type { DashboardData, Invoice } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import PeriodStepper from '@/components/ui/PeriodStepper.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import InvoiceSeal from '@/components/ui/InvoiceSeal.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import MonthlyBars from '@/components/dashboard/MonthlyBars.vue'
import ThresholdMeter from '@/components/dashboard/ThresholdMeter.vue'
import TaxSplit from '@/components/dashboard/TaxSplit.vue'
import { useConfigStore } from '@/stores/config'
import { errorMessage } from '@/stores/toast'
import { formatCurrency, formatDateShort, splitCurrency, ITALIAN_MONTHS } from '@/utils/format'
import { estimateForfettarioTax, estimateOrdinarioTax } from '@/utils/tax'
import { summariseAttention, type Attention } from '@/utils/attention'
import { plural } from '@/utils/labels'

const router = useRouter()
const configStore = useConfigStore()

const now = new Date()
const currentYear = now.getFullYear()
const currentMonth = now.getMonth() + 1

const selectedYear = ref(currentYear)
const data = ref<DashboardData | null>(null)
const yearInvoices = ref<Invoice[]>([])
const unbilled = ref<{ month: number; year: number; sessions: number; amount: number } | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const firstFiveYears = ref(false)

const greeting = computed(() => {
  const hour = now.getHours()
  const name = configStore.config?.first_name
  const salutation = hour < 13 ? 'Buongiorno' : hour < 18 ? 'Buon pomeriggio' : 'Buonasera'
  return name ? `${salutation}, ${name}` : salutation
})

/**
 * The sessions of last month that were held but never invoiced: the most
 * common thing left undone at the start of a month.
 */
async function loadUnbilled(): Promise<void> {
  unbilled.value = null
  if (selectedYear.value !== currentYear) return
  const month = currentMonth === 1 ? 12 : currentMonth - 1
  const year = currentMonth === 1 ? currentYear - 1 : currentYear
  const previews = await previewMonthlyInvoices(year, month)
  const sessions = previews.reduce((sum, preview) => sum + preview.appointment_count, 0)
  if (sessions === 0) return
  const amount = previews.reduce((sum, preview) => sum + preview.estimated_due, 0)
  unbilled.value = { month, year, sessions, amount }
}

async function load(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const [dashboard, invoices] = await Promise.all([
      getDashboard(selectedYear.value),
      listInvoices({ year: selectedYear.value }),
      loadUnbilled(),
    ])
    data.value = dashboard
    yearInvoices.value = invoices
  } catch (cause) {
    error.value = errorMessage(cause)
    data.value = null
  } finally {
    loading.value = false
  }
}

function shiftYear(delta: number): void {
  selectedYear.value += delta
  void load()
}

const hero = computed(() => splitCurrency(data.value?.total_revenue ?? 0))
const collectionRate = computed(() => {
  if (!data.value || data.value.total_revenue === 0) return 0
  return Math.round((data.value.paid_revenue / data.value.total_revenue) * 100)
})

const isForfettario = computed(() => configStore.config?.tax_regime !== 'ordinario')

const taxEstimate = computed(() => {
  if (!data.value || !configStore.config) return null
  const revenue = data.value.total_net_revenue
  if (revenue <= 0) return null
  const { coefficient } = configStore.config
  if (isForfettario.value) {
    const estimate = estimateForfettarioTax(revenue, coefficient, firstFiveYears.value)
    return { revenue, net: estimate.netIncome, contributions: estimate.inpsContribution, tax: estimate.substituteTax, taxLabel: `Imposta sostitutiva ${estimate.substituteTaxRate}%` }
  }
  const estimate = estimateOrdinarioTax(revenue, coefficient)
  return {
    revenue,
    net: estimate.netIncome,
    contributions: estimate.inpsContribution,
    tax: estimate.irpef + estimate.addizionaleRegionale + estimate.addizionaleComunale,
    taxLabel: 'IRPEF e addizionali',
  }
})

const attention = computed<Attention>(() => summariseAttention(yearInvoices.value))
const nothingToDo = computed(
  () => attention.value.overdue.count === 0 && attention.value.drafts.count === 0 && unbilled.value === null,
)

const RATE_OPTIONS = [
  { value: true, label: '5% (primi 5 anni)' },
  { value: false, label: '15%' },
] as const

onMounted(load)
</script>

<template>
  <div>
    <PageHeader :title="`Panoramica ${selectedYear}`" :icon="LayoutGrid">
      <template #eyebrow>{{ greeting }}</template>
      <PeriodStepper
        :label="String(selectedYear)"
        previous-label="Anno precedente"
        next-label="Anno successivo"
        :can-next="selectedYear < currentYear"
        @previous="shiftYear(-1)"
        @next="shiftYear(1)"
      />
    </PageHeader>

    <div class="page pt-6 pb-12">
      <!-- Loading: the same shapes the page will have. -->
      <div v-if="loading" class="space-y-5" role="status" aria-busy="true" aria-live="polite">
        <span class="sr-only">Caricamento della panoramica</span>
        <div class="grid grid-cols-[18rem_1fr] gap-5 rounded-card border border-border bg-surface-raised p-6">
          <div class="space-y-3"><div class="skeleton h-3 w-24" /><div class="skeleton h-12 w-48" /><div class="skeleton h-3 w-32" /></div>
          <div class="skeleton h-48" />
        </div>
        <div class="grid grid-cols-3 gap-5">
          <div v-for="index in 3" :key="index" class="h-52 rounded-card border border-border bg-surface-raised p-5"><div class="skeleton h-3 w-24" /></div>
        </div>
      </div>

      <EmptyState
        v-else-if="error"
        :icon="TriangleAlert"
        title="Non è stato possibile caricare la panoramica"
        :description="error"
      >
        <AppButton @click="load">Riprova</AppButton>
      </EmptyState>

      <div v-else-if="data" class="space-y-5">
        <!-- ── The year in one sheet: the hero figure and its months ─────────── -->
        <AppCard class="settle grid grid-cols-1 lg:grid-cols-[19rem_1fr]" :padded="false">
          <div class="hero-figure relative flex flex-col overflow-hidden rounded-l-card p-6 lg:pr-6">
            <p class="label-quiet flex items-center gap-2">
              <span class="icon-chip icon-chip-sm" aria-hidden="true"><Receipt :size="13" :stroke-width="1.9" /></span>
              Fatturato {{ selectedYear }}
            </p>
            <p class="mt-3 whitespace-nowrap font-semibold tracking-[-0.025em] text-text" aria-live="polite">
              <span class="text-[3.25rem] leading-none">{{ hero.whole }}</span><span class="text-2xl text-text-muted">{{ hero.fraction }}</span>
              <span class="ml-1 text-2xl text-text-subtle">{{ hero.symbol }}</span>
            </p>
            <p class="mt-2 text-sm text-text-muted">
              {{ plural(data.total_invoices, 'fattura', 'fatture') }}<template v-if="data.draft_invoices > 0">, di cui {{ plural(data.draft_invoices, 'bozza', 'bozze') }}</template>
            </p>

            <dl class="mt-auto space-y-4 pt-8">
              <div>
                <div class="flex items-baseline justify-between">
                  <dt class="label-quiet">Incassato</dt>
                  <dd class="tabular text-xs text-text-subtle">{{ collectionRate }}%</dd>
                </div>
                <dd class="tabular mt-0.5 text-lg font-semibold text-text">{{ formatCurrency(data.paid_revenue) }}</dd>
                <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-safe/15">
                  <div class="h-full rounded-full bg-safe transition-[width] duration-700 ease-out-expo" :style="{ width: `${collectionRate}%` }" />
                </div>
              </div>
              <div>
                <dt class="label-quiet">Da incassare</dt>
                <dd class="tabular mt-0.5 text-lg font-semibold text-text">{{ formatCurrency(data.unpaid_revenue) }}</dd>
              </div>
            </dl>
          </div>

          <div class="flex flex-col border-t border-border lg:border-t-0 lg:border-l">
            <CardHeader title="Incassato per mese" subtitle="Fatture pagate, per mese di emissione" :icon="BarChart3" />
            <MonthlyBars
              class="flex-1 px-5 pb-5"
              :months="data.monthly_revenue" :current-month="selectedYear === currentYear ? currentMonth : null" />
          </div>
        </AppCard>

        <!-- ── Three instruments: ceiling, tax, to-do ──────────────────────── -->
        <div class="grid grid-cols-1 gap-5 md:grid-cols-2 xl:grid-cols-3">
          <AppCard v-if="isForfettario" class="settle" :padded="false" style="--settle: 1">
            <CardHeader title="Soglia forfettario" :subtitle="`Compensi fatturati nel ${selectedYear}`" :icon="Gauge" />
            <div class="px-5 pb-5">
              <ThresholdMeter :amount="data.total_net_revenue" :year="selectedYear" />
            </div>
          </AppCard>

          <AppCard class="settle flex flex-col" :padded="false" style="--settle: 2">
            <CardHeader title="Stima fiscale" :subtitle="`Su ${formatCurrency(data.total_net_revenue)} di compensi`" :icon="Calculator" tone="safe" />
            <div class="flex flex-1 flex-col px-5 pb-5">
              <template v-if="taxEstimate">
                <TaxSplit v-bind="taxEstimate" />
                <div v-if="isForfettario" class="mt-5">
                  <SegmentedControl v-model="firstFiveYears" :options="RATE_OPTIONS" label="Aliquota imposta sostitutiva" size="sm" block />
                </div>
                <p class="mt-auto pt-4 text-2xs text-text-subtle">Stima indicativa: non sostituisce il parere del commercialista.</p>
              </template>
              <p v-else class="text-sm text-text-muted">La stima appare con la prima fattura dell'anno.</p>
            </div>
          </AppCard>

          <AppCard class="settle flex flex-col" :padded="false" style="--settle: 3">
            <CardHeader title="Da fare" subtitle="Quello che aspetta te" :icon="ListChecks" :tone="attention.overdue.count > 0 ? 'danger' : 'accent'" />
            <div v-if="nothingToDo" class="flex flex-1 flex-col items-center justify-center px-5 pb-8 pt-4 text-center">
              <CircleCheck :size="28" :stroke-width="1.5" class="text-safe" aria-hidden="true" />
              <p class="mt-2 text-base font-medium text-text">Tutto in ordine</p>
              <p class="text-sm text-text-muted">Nessuna scadenza superata, nessuna bozza in sospeso.</p>
            </div>
            <ul v-else class="px-2 pb-2">
              <li v-if="unbilled">
                <RouterLink
                  :to="{ path: '/invoices/monthly', query: { year: unbilled.year, month: unbilled.month } }"
                  class="group flex items-center gap-3 rounded-control px-3 py-2.5 transition-colors hover:bg-surface-hover focus-ring"
                >
                  <span class="icon-chip" style="--chip: var(--accent)"><CalendarRange :size="16" :stroke-width="1.75" aria-hidden="true" /></span>
                  <span class="min-w-0 flex-1">
                    <span class="block text-base font-medium text-text">Fattura le sedute di {{ ITALIAN_MONTHS[unbilled.month - 1] }}</span>
                    <span class="block text-sm text-text-muted">{{ plural(unbilled.sessions, 'seduta svolta', 'sedute svolte') }} · {{ formatCurrency(unbilled.amount) }}</span>
                  </span>
                  <ArrowRight :size="15" class="text-text-subtle transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
                </RouterLink>
              </li>
              <li v-if="attention.overdue.count > 0">
                <RouterLink
                  :to="{ path: '/invoices', query: { status: 'overdue', year: selectedYear } }"
                  class="group flex items-center gap-3 rounded-control px-3 py-2.5 transition-colors hover:bg-surface-hover focus-ring"
                >
                  <span class="icon-chip" style="--chip: var(--danger)"><TriangleAlert :size="16" :stroke-width="1.75" aria-hidden="true" /></span>
                  <span class="min-w-0 flex-1">
                    <span class="block text-base font-medium text-text">{{ plural(attention.overdue.count, 'fattura scaduta', 'fatture scadute') }}</span>
                    <span class="block text-sm text-text-muted">{{ formatCurrency(attention.overdue.amount) }} da sollecitare</span>
                  </span>
                  <ArrowRight :size="15" class="text-text-subtle transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
                </RouterLink>
              </li>
              <li v-if="attention.drafts.count > 0">
                <RouterLink
                  :to="{ path: '/invoices', query: { status: 'draft', year: selectedYear } }"
                  class="group flex items-center gap-3 rounded-control px-3 py-2.5 transition-colors hover:bg-surface-hover focus-ring"
                >
                  <span class="icon-chip" style="--chip: var(--text-muted)"><FilePen :size="16" :stroke-width="1.75" aria-hidden="true" /></span>
                  <span class="min-w-0 flex-1">
                    <span class="block text-base font-medium text-text">{{ plural(attention.drafts.count, 'bozza da emettere', 'bozze da emettere') }}</span>
                    <span class="block text-sm text-text-muted">{{ formatCurrency(attention.drafts.amount) }} in totale</span>
                  </span>
                  <ArrowRight :size="15" class="text-text-subtle transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
                </RouterLink>
              </li>
              <li v-if="attention.awaiting.count > 0">
                <RouterLink
                  :to="{ path: '/invoices', query: { status: 'issued', year: selectedYear } }"
                  class="group flex items-center gap-3 rounded-control px-3 py-2.5 transition-colors hover:bg-surface-hover focus-ring"
                >
                  <span class="icon-chip" style="--chip: var(--warn)"><Hourglass :size="16" :stroke-width="1.75" aria-hidden="true" /></span>
                  <span class="min-w-0 flex-1">
                    <span class="block text-base font-medium text-text">{{ plural(attention.awaiting.count, 'fattura in attesa', 'fatture in attesa') }}</span>
                    <span class="block text-sm text-text-muted">{{ formatCurrency(attention.awaiting.amount) }} non ancora scadute</span>
                  </span>
                  <ArrowRight :size="15" class="text-text-subtle transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
                </RouterLink>
              </li>
            </ul>
          </AppCard>
        </div>

        <!-- ── Recent invoices ─────────────────────────────────────────────── -->
        <AppCard class="settle" :padded="false" style="--settle: 4">
          <CardHeader title="Fatture recenti" :subtitle="`Le ultime emesse nel ${selectedYear}`" :icon="FileText">
            <AppButton variant="ghost" size="sm" :icon-right="ArrowRight" to="/invoices">Tutte le fatture</AppButton>
          </CardHeader>

          <EmptyState
            v-if="data.recent_invoices.length === 0"
            :icon="FileText"
            :bordered="false"
            :title="`Nessuna fattura nel ${selectedYear}`"
            description="Le fatture che emetti compariranno qui."
          >
            <AppButton variant="primary" to="/invoices/new">Nuova fattura</AppButton>
          </EmptyState>

          <ul v-else class="border-t border-border">
            <li v-for="invoice in data.recent_invoices" :key="invoice.id" class="border-b border-border last:border-b-0">
              <button
                type="button"
                class="grid h-row w-full grid-cols-[1.25rem_5.5rem_1fr_4.5rem_7rem_6.5rem] items-center gap-4 px-5 text-left transition-[background-color,box-shadow] hover:bg-surface-hover hover:shadow-[inset_2px_0_0_var(--accent)] focus-ring"
                @click="router.push(`/invoices/${invoice.id}`)"
              >
                <InvoiceSeal :status="invoice.status" :issue-date="invoice.issue_date" :due-date="invoice.due_date" :size="18" />
                <span class="tabular text-sm text-text-muted">N. {{ invoice.invoice_number }}</span>
                <span class="flex min-w-0 items-center gap-2.5">
                  <PatientMonogram :name="invoice.client_name" size="sm" />
                  <span class="truncate text-base text-text">{{ invoice.client_name }}</span>
                </span>
                <span class="text-sm text-text-subtle">{{ formatDateShort(invoice.issue_date) }}</span>
                <span class="tabular text-right text-base font-medium text-text">{{ formatCurrency(invoice.total_due) }}</span>
                <span class="flex justify-end"><StatusBadge type="invoice" :status="invoice.status" /></span>
              </button>
            </li>
          </ul>
        </AppCard>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* The year's figure sits on a faint wash of ink, rising from the bottom corner like a watermark. */
.hero-figure {
  background:
    radial-gradient(120% 70% at 0% 100%, color-mix(in srgb, var(--accent) 8%, transparent), transparent 70%),
    linear-gradient(180deg, transparent, color-mix(in srgb, var(--surface-sunken) 35%, transparent));
}
</style>
