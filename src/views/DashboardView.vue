<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { TrendingUp, Euro, Clock, FileText, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { getDashboard } from '@/api'
import type { DashboardData, MonthlyRevenue } from '@/types'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { useConfigStore } from '@/stores/config'
import { formatCurrency, formatDate } from '@/utils/format'
import { estimateForfettarioTax, estimateOrdinarioTax } from '@/utils/tax'

const router = useRouter()
const configStore = useConfigStore()
const currentYear  = new Date().getFullYear()
const currentMonth = new Date().getMonth() + 1   // 1–12
const selectedYear = ref(currentYear)
const data         = ref<DashboardData | null>(null)
const loading      = ref(false)
const error        = ref<string | null>(null)
const firstFiveYears = ref(false)

/** Month currently hovered in the chart (1–12), null when none. */
const hoveredMonth = ref<number | null>(null)

async function loadDashboard() {
  loading.value = true
  error.value   = null
  try {
    data.value = await getDashboard(selectedYear.value)
  } catch (e) {
    error.value = String(e)
    data.value  = null
  } finally {
    loading.value = false
  }
}

function prevYear() { selectedYear.value--; loadDashboard() }
function nextYear() { selectedYear.value++; loadDashboard() }

const maxRevenue = computed(() => {
  if (!data.value) return 1
  return Math.max(...data.value.monthly_revenue.map(m => m.revenue), 1)
})

function barHeight(revenue: number): string {
  if (revenue === 0) return '2%'
  return `${Math.max((revenue / maxRevenue.value) * 100, 5)}%`
}

function isCurrentMonth(month: number): boolean {
  return selectedYear.value === currentYear && month === currentMonth
}

/** Returns the gradient string for a bar based on its state. */
function barGradient(month: MonthlyRevenue): string {
  if (month.revenue === 0) return 'rgba(163,186,163,0.15)'
  const hovered = hoveredMonth.value === month.month
  if (isCurrentMonth(month.month)) {
    return hovered
      ? 'linear-gradient(to top, #0153a2, #0c8aeb, #7cc2fd)'
      : 'linear-gradient(to top, #0153a2, #0c8aeb, #36a5fa)'
  }
  return hovered
    ? 'linear-gradient(to top, #3a513e, #5d8062, #a1baa3)'
    : 'linear-gradient(to top, #3a513e, #5d8062, #7a9b7e)'
}

const peakMonth = computed((): MonthlyRevenue | null => {
  if (!data.value) return null
  const months = data.value.monthly_revenue.filter(m => m.revenue > 0)
  if (!months.length) return null
  return months.reduce((a, b) => a.revenue > b.revenue ? a : b)
})

const isForfettario = computed(() => configStore.config?.tax_regime === 'forfettario')

const taxEstimate = computed(() => {
  if (!data.value || !configStore.config) return null
  const revenue     = data.value.total_net_revenue
  if (revenue <= 0) return null
  const { coefficient } = configStore.config
  if (isForfettario.value) {
    return { type: 'forfettario' as const, ...estimateForfettarioTax(revenue, coefficient, firstFiveYears.value) }
  }
  return { type: 'ordinario' as const, ...estimateOrdinarioTax(revenue, coefficient) }
})

const netPercentage = computed(() => {
  if (!taxEstimate.value || taxEstimate.value.annualRevenue === 0) return 100
  return Math.round((taxEstimate.value.netIncome / taxEstimate.value.annualRevenue) * 100)
})

onMounted(loadDashboard)
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">

      <!-- ── Header ──────────────────────────────────────────────────────── -->
      <div class="flex items-center justify-between mb-8 animate-in">
        <div>
          <h1 class="text-2xl font-semibold text-sage-900 tracking-tight">Dashboard</h1>
          <p class="text-sm text-sage-500 mt-0.5">Panoramica anno fiscale</p>
        </div>
        <div class="glass-card flex items-center gap-1 rounded-xl px-1 py-1 shadow-sm">
          <button
            type="button"
            class="p-1.5 text-sage-400 hover:text-sage-700 rounded-lg hover:bg-sage-50/60 transition-all"
            @click="prevYear"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <span class="text-sm font-semibold text-sage-800 min-w-[3rem] text-center px-1">
            {{ selectedYear }}
          </span>
          <button
            type="button"
            class="p-1.5 text-sage-400 hover:text-sage-700 rounded-lg hover:bg-sage-50/60 transition-all"
            @click="nextYear"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- ── Loading ─────────────────────────────────────────────────────── -->
      <div v-if="loading" class="flex items-center justify-center h-64">
        <div class="flex flex-col items-center gap-3">
          <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
          <p class="text-sm text-sage-400">Caricamento...</p>
        </div>
      </div>

      <!-- ── Error ───────────────────────────────────────────────────────── -->
      <div v-else-if="error" class="flex items-center justify-center h-64">
        <div class="glass-card rounded-2xl px-6 py-5 max-w-md text-center shadow-sm">
          <p class="text-sm font-semibold text-red-700 mb-1">Errore nel caricamento</p>
          <p class="text-xs text-red-500 font-mono break-all">{{ error }}</p>
          <button
            type="button"
            class="mt-4 bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-1.5 rounded-lg text-sm font-medium"
            @click="loadDashboard"
          >
            Riprova
          </button>
        </div>
      </div>

      <template v-else-if="data">

        <!-- ── KPI cards ───────────────────────────────────────────────── -->
        <div class="grid grid-cols-4 gap-4 mb-5">
          <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d1">
            <div class="flex items-center justify-between mb-4">
              <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Fatturato totale</span>
              <div class="w-9 h-9 rounded-xl flex items-center justify-center" style="background: linear-gradient(135deg, #5d8062, #48654c)">
                <TrendingUp class="w-4 h-4 text-white" />
              </div>
            </div>
            <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ formatCurrency(data.total_revenue) }}</p>
            <p class="text-xs text-sage-400 mt-1.5">{{ data.total_invoices }} fatture totali</p>
          </div>

          <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d2">
            <div class="flex items-center justify-between mb-4">
              <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Incassato</span>
              <div class="w-9 h-9 rounded-xl flex items-center justify-center" style="background: linear-gradient(135deg, #0c8aeb, #0153a2)">
                <Euro class="w-4 h-4 text-white" />
              </div>
            </div>
            <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ formatCurrency(data.paid_revenue) }}</p>
            <p class="text-xs text-sage-400 mt-1.5">{{ data.paid_invoices }} fatture pagate</p>
          </div>

          <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d3">
            <div class="flex items-center justify-between mb-4">
              <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Da incassare</span>
              <div class="w-9 h-9 rounded-xl flex items-center justify-center" style="background: linear-gradient(135deg, #d4a017, #a16207)">
                <Clock class="w-4 h-4 text-white" />
              </div>
            </div>
            <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ formatCurrency(data.unpaid_revenue) }}</p>
            <p class="text-xs text-sage-400 mt-1.5">{{ data.draft_invoices }} bozze</p>
          </div>

          <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d4">
            <div class="flex items-center justify-between mb-4">
              <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">N. Fatture</span>
              <div class="w-9 h-9 rounded-xl flex items-center justify-center" style="background: linear-gradient(135deg, #b88e67, #8a5f42)">
                <FileText class="w-4 h-4 text-white" />
              </div>
            </div>
            <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ data.total_invoices }}</p>
            <p class="text-xs text-sage-400 mt-1.5">nell'anno {{ selectedYear }}</p>
          </div>
        </div>

        <!-- ── Chart + Tax ─────────────────────────────────────────────── -->
        <div class="grid grid-cols-5 gap-4 mb-5">

          <!-- ── Monthly bar chart ───────────────────────────────────── -->
          <div class="col-span-3 glass-card rounded-2xl p-5 shadow-sm animate-in-d2 flex flex-col gap-4">

            <!-- Card header -->
            <div class="flex items-start justify-between">
              <div>
                <h2 class="text-sm font-semibold text-sage-800">Andamento mensile</h2>
                <p class="text-xs text-sage-400 mt-0.5">Fatturato netto · {{ selectedYear }}</p>
              </div>
              <!-- Peak month badge -->
              <div v-if="peakMonth" class="shrink-0 ml-4 text-right">
                <p class="text-[10px] uppercase tracking-wider text-sage-400">Mese migliore</p>
                <p class="text-xs font-semibold text-sage-700 mt-0.5">
                  {{ peakMonth.month_name }} &middot; {{ formatCurrency(peakMonth.revenue) }}
                </p>
              </div>
            </div>

            <!-- Chart body -->
            <div class="flex flex-col gap-2">

              <!--
                Bar area: FIXED height h-44.
                Guide lines via absolute overlay at z-index:-1 (renders behind
                non-positioned flex children in the same stacking context).
                overflow-visible lets tooltips escape upward.
              -->
              <div class="h-44 flex items-end gap-1 relative overflow-visible">

                <!-- Guide lines (z:-1 → behind bars) -->
                <div
                  class="absolute inset-0 flex flex-col justify-between pointer-events-none"
                  style="z-index: -1"
                >
                  <div class="h-px w-full bg-sage-200/40" />
                  <div class="h-px w-full bg-sage-200/35" />
                  <div class="h-px w-full bg-sage-200/30" />
                  <div class="h-px w-full bg-sage-200/25" />
                  <div /><!-- spacer aligns to baseline -->
                </div>

                <!--
                  Bar columns: relative so the absolute tooltip positions
                  relative to each column. No z-index → no stacking context
                  → tooltip z-50 escapes to the document root SC.
                -->
                <div
                  v-for="month in data.monthly_revenue"
                  :key="month.month"
                  class="flex-1 h-full flex items-end relative rounded-sm transition-colors duration-150 cursor-default"
                  :class="hoveredMonth === month.month ? 'bg-sage-50/50' : 'bg-transparent'"
                  @mouseenter="hoveredMonth = month.month"
                  @mouseleave="hoveredMonth = null"
                >
                  <!-- Tooltip (appears above the bar top; Vue-controlled for clean z-index) -->
                  <div
                    v-if="hoveredMonth === month.month && month.revenue > 0"
                    class="absolute left-1/2 -translate-x-1/2 bottom-[calc(100%+8px)]
                           bg-sage-900/95 text-white rounded-lg px-2.5 py-1.5
                           whitespace-nowrap shadow-xl pointer-events-none z-50"
                    style="font-size: 11px; font-weight: 500; letter-spacing: -0.01em"
                  >
                    {{ formatCurrency(month.revenue) }}
                    <span class="text-sage-400 ml-1.5 font-normal" style="font-size: 10px">
                      {{ month.invoice_count }} fatt.
                    </span>
                    <!-- Arrow -->
                    <span
                      class="absolute left-1/2 -translate-x-1/2 top-full block"
                      style="width:0;height:0;border-left:5px solid transparent;border-right:5px solid transparent;border-top:5px solid rgba(40,55,43,0.95)"
                    />
                  </div>

                  <!-- Bar -->
                  <div
                    class="w-full rounded-t-[4px] transition-all duration-500"
                    :style="{
                      height: barHeight(month.revenue),
                      background: barGradient(month),
                    }"
                  />
                </div>
              </div>

              <!-- Month labels row (fixed below bars, never affects bar heights) -->
              <div class="flex gap-1">
                <div
                  v-for="month in data.monthly_revenue"
                  :key="month.month"
                  class="flex-1 text-center"
                >
                  <span
                    class="text-[10px] transition-colors duration-150"
                    :class="isCurrentMonth(month.month)
                      ? 'text-ocean-500 font-semibold'
                      : hoveredMonth === month.month
                        ? 'text-sage-600'
                        : 'text-sage-400'"
                  >
                    {{ month.month_name.slice(0, 3) }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- ── Tax estimate ────────────────────────────────────────── -->
          <div class="col-span-2 glass-card rounded-2xl p-5 shadow-sm animate-in-d2 flex flex-col">
            <div class="flex items-center justify-between mb-3">
              <h2 class="text-sm font-semibold text-sage-800">Stima Fiscale</h2>
              <span
                class="text-[10px] font-semibold px-2.5 py-0.5 rounded-full uppercase tracking-wider"
                :class="isForfettario
                  ? 'bg-sage-100/80 text-sage-600'
                  : 'bg-blue-50 text-blue-600'"
              >
                {{ isForfettario ? 'Forfettario' : 'Ordinario' }}
              </span>
            </div>

            <template v-if="taxEstimate">
              <!-- Compensi -->
              <div class="mb-2.5">
                <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Compensi</p>
                <p class="text-xl font-bold text-sage-900 tracking-tight">
                  {{ formatCurrency(data.total_net_revenue) }}
                </p>
              </div>

              <!-- Proportion bar -->
              <div class="mb-3.5">
                <div class="h-1.5 rounded-full overflow-hidden flex bg-sage-50">
                  <div
                    class="rounded-l-full transition-all duration-700"
                    :style="{ width: netPercentage + '%', background: 'linear-gradient(90deg, #48654c, #5d8062)' }"
                  />
                  <div
                    class="rounded-r-full transition-all duration-700"
                    :style="{ width: (100 - netPercentage) + '%', background: 'linear-gradient(90deg, #d4a017, #a16207)' }"
                  />
                </div>
                <div class="flex justify-between mt-1">
                  <span class="text-[9px] text-sage-400">{{ netPercentage }}% netto</span>
                  <span class="text-[9px] text-amber-500/80">{{ 100 - netPercentage }}% imposte</span>
                </div>
              </div>

              <!-- Breakdown -->
              <div class="space-y-1.5 flex-1">
                <div class="flex justify-between text-xs">
                  <span class="text-sage-500">
                    Redd. imponibile
                    <span class="text-sage-300">({{ configStore.config?.coefficient }}%)</span>
                  </span>
                  <span class="text-sage-600 font-medium">{{ formatCurrency(taxEstimate.taxableIncome) }}</span>
                </div>
                <div class="flex justify-between text-xs">
                  <span class="text-sage-500">Contributi prev.</span>
                  <span class="text-amber-600 font-medium">&minus;{{ formatCurrency(taxEstimate.inpsContribution) }}</span>
                </div>

                <template v-if="taxEstimate.type === 'forfettario'">
                  <div class="flex justify-between text-xs">
                    <span class="text-sage-500">
                      Imp. sostitutiva
                      <span class="text-sage-300">({{ taxEstimate.substituteTaxRate }}%)</span>
                    </span>
                    <span class="text-amber-600 font-medium">&minus;{{ formatCurrency(taxEstimate.substituteTax) }}</span>
                  </div>
                </template>

                <template v-else>
                  <div class="flex justify-between text-xs">
                    <span class="text-sage-500">IRPEF</span>
                    <span class="text-amber-600 font-medium">&minus;{{ formatCurrency(taxEstimate.irpef) }}</span>
                  </div>
                  <div class="flex justify-between text-xs">
                    <span class="text-sage-500">Addizionali</span>
                    <span class="text-amber-600 font-medium">
                      &minus;{{ formatCurrency(taxEstimate.addizionaleRegionale + taxEstimate.addizionaleComunale) }}
                    </span>
                  </div>
                </template>
              </div>

              <!-- Net result -->
              <div class="border-t border-sage-100/60 pt-2.5 mt-2.5">
                <div class="flex justify-between items-baseline">
                  <span class="text-xs font-semibold text-sage-700">Netto stimato</span>
                  <span class="text-base font-bold text-sage-900">{{ formatCurrency(taxEstimate.netIncome) }}</span>
                </div>
              </div>

              <!-- Tax rate toggle (forfettario only) -->
              <div v-if="isForfettario" class="flex items-center gap-1.5 mt-2.5 pt-2.5 border-t border-sage-50">
                <span class="text-[10px] text-sage-400 mr-1">Aliquota</span>
                <button
                  type="button"
                  class="text-[10px] px-2 py-0.5 rounded-full transition-all"
                  :class="firstFiveYears ? 'bg-sage-600 text-white' : 'bg-sage-50 text-sage-400 hover:text-sage-600'"
                  @click="firstFiveYears = true"
                >5%</button>
                <button
                  type="button"
                  class="text-[10px] px-2 py-0.5 rounded-full transition-all"
                  :class="!firstFiveYears ? 'bg-sage-600 text-white' : 'bg-sage-50 text-sage-400 hover:text-sage-600'"
                  @click="firstFiveYears = false"
                >15%</button>
              </div>
            </template>

            <!-- Empty state -->
            <div v-else class="flex-1 flex items-center justify-center">
              <div class="text-center py-4">
                <Euro class="w-8 h-8 text-sage-200 mx-auto mb-1.5" />
                <p class="text-xs text-sage-400">Nessun compenso registrato</p>
              </div>
            </div>
          </div>
        </div>

        <!-- ── Recent invoices ─────────────────────────────────────────── -->
        <div class="glass-card rounded-2xl shadow-sm animate-in-d3">
          <div class="px-6 py-4 flex items-center justify-between border-b border-sage-100/50">
            <h2 class="text-sm font-semibold text-sage-800">Fatture recenti</h2>
            <button
              type="button"
              class="text-xs font-semibold text-sage-500 hover:text-sage-700 transition-colors"
              @click="router.push('/invoices')"
            >
              Vedi tutte →
            </button>
          </div>

          <div v-if="data.recent_invoices.length === 0" class="px-6 py-10 text-center">
            <FileText class="w-10 h-10 text-sage-200 mx-auto mb-2" />
            <p class="text-sm text-sage-400">Nessuna fattura nell'anno.</p>
          </div>

          <table v-else class="w-full text-sm">
            <thead>
              <tr class="border-b border-sage-50">
                <th class="px-6 py-3 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Numero</th>
                <th class="px-6 py-3 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Cliente</th>
                <th class="px-6 py-3 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Data</th>
                <th class="px-6 py-3 text-right text-xs font-semibold text-sage-400 uppercase tracking-wider">Totale</th>
                <th class="px-6 py-3 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Stato</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="invoice in data.recent_invoices"
                :key="invoice.id"
                class="border-b border-sage-50/70 hover:bg-sage-50/40 cursor-pointer transition-colors"
                @click="router.push(`/invoices/${invoice.id}`)"
              >
                <td class="px-6 py-3.5 font-semibold text-sage-800">{{ invoice.invoice_number }}</td>
                <td class="px-6 py-3.5 text-sage-600">{{ invoice.client_name }}</td>
                <td class="px-6 py-3.5 text-sage-400">{{ formatDate(invoice.issue_date) }}</td>
                <td class="px-6 py-3.5 text-right font-semibold text-sage-800">{{ formatCurrency(invoice.total_due) }}</td>
                <td class="px-6 py-3.5"><StatusBadge :status="invoice.status" type="invoice" /></td>
              </tr>
            </tbody>
          </table>
        </div>

      </template>

      <div v-else class="flex items-center justify-center h-64">
        <div class="text-center">
          <p class="text-sm text-sage-400">Nessun dato disponibile.</p>
          <button
            type="button"
            class="mt-3 text-sm font-medium text-sage-600 hover:text-sage-800 transition-colors"
            @click="loadDashboard"
          >
            Ricarica
          </button>
        </div>
      </div>

    </div>
  </div>
</template>
