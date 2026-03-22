<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  TrendingUp, Euro, Clock, FileText,
  ChevronLeft, ChevronRight,
  Plus, Users, CalendarDays, ArrowRight,
  CheckCircle2,
} from 'lucide-vue-next'
import { getDashboard } from '@/api'
import type { DashboardData, MonthlyRevenue } from '@/types'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { useConfigStore } from '@/stores/config'
import { formatCurrency, formatDate } from '@/utils/format'
import { estimateForfettarioTax, estimateOrdinarioTax } from '@/utils/tax'

const router = useRouter()
const configStore = useConfigStore()
const currentYear  = new Date().getFullYear()
const currentMonth = new Date().getMonth() + 1
const currentHour  = new Date().getHours()
const selectedYear = ref(currentYear)
const data         = ref<DashboardData | null>(null)
const loading      = ref(false)
const error        = ref<string | null>(null)
const firstFiveYears = ref(false)
const hoveredMonth = ref<number | null>(null)

const greeting = computed(() => {
  if (currentHour < 12) return 'Buongiorno'
  if (currentHour < 18) return 'Buon pomeriggio'
  return 'Buonasera'
})

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
  const revenue = data.value.total_net_revenue
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

const collectionRate = computed(() => {
  if (!data.value || data.value.total_revenue === 0) return 0
  return Math.round((data.value.paid_revenue / data.value.total_revenue) * 100)
})

const issuedUnpaid = computed(() => {
  if (!data.value) return 0
  return Math.max(0, data.value.total_invoices - data.value.paid_invoices - data.value.draft_invoices)
})

const AVATAR_GRADIENTS = [
  'linear-gradient(135deg, #5d8062, #48654c)',
  'linear-gradient(135deg, #0c8aeb, #0153a2)',
  'linear-gradient(135deg, #b88e67, #8a5f42)',
  'linear-gradient(135deg, #7a9b7e, #5d8062)',
  'linear-gradient(135deg, #d4a017, #a16207)',
]

function clientInitials(name: string): string {
  return name.split(' ').slice(0, 2).map(w => w[0] ?? '').join('').toUpperCase()
}

function clientAvatarGradient(id: number): string {
  return AVATAR_GRADIENTS[id % AVATAR_GRADIENTS.length]
}

onMounted(loadDashboard)
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">

      <!-- ── Header ──────────────────────────────────────────────────────── -->
      <div class="flex items-start justify-between mb-8 animate-in">
        <div>
          <p class="text-[10px] font-bold text-sage-400 uppercase tracking-widest mb-1">{{ greeting }}</p>
          <h1 class="text-2xl font-bold text-sage-900 tracking-tight">
            {{ configStore.config?.first_name || 'Dashboard' }}
          </h1>
          <p class="text-sm text-sage-500 mt-0.5">Panoramica anno fiscale {{ selectedYear }}</p>
        </div>
        <!-- Year picker -->
        <div class="glass-card flex items-center gap-0.5 rounded-xl px-1.5 py-1.5 shadow-sm">
          <button
            type="button"
            class="p-1.5 text-sage-400 hover:text-sage-700 rounded-lg hover:bg-sage-50/60 transition-all cursor-pointer"
            @click="prevYear"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <span class="text-sm font-bold text-sage-800 min-w-[3.5rem] text-center tabular-nums">
            {{ selectedYear }}
          </span>
          <button
            type="button"
            class="p-1.5 text-sage-400 hover:text-sage-700 rounded-lg hover:bg-sage-50/60 transition-all cursor-pointer"
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
            class="mt-4 bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-1.5 rounded-lg text-sm font-medium cursor-pointer"
            @click="loadDashboard"
          >
            Riprova
          </button>
        </div>
      </div>

      <template v-else-if="data">

        <!-- ── KPI cards ───────────────────────────────────────────────── -->
        <div class="grid grid-cols-4 gap-4 mb-4">

          <!-- Fatturato totale -->
          <div class="glass-card rounded-2xl overflow-hidden shadow-sm hover-lift animate-in-d1 cursor-default">
            <div class="h-0.5 w-full" style="background: linear-gradient(90deg, #5d8062, #48654c)" />
            <div class="p-5">
              <div class="flex items-center justify-between mb-3">
                <span class="text-[10px] font-bold text-sage-400 uppercase tracking-widest">Fatturato</span>
                <div class="w-8 h-8 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #5d8062, #48654c)">
                  <TrendingUp class="w-3.5 h-3.5 text-white" />
                </div>
              </div>
              <p class="text-2xl font-bold text-sage-900 tracking-tight tabular-nums">{{ formatCurrency(data.total_revenue) }}</p>
              <p class="text-[11px] text-sage-400 mt-1.5">{{ data.total_invoices }} fatture emesse</p>
            </div>
          </div>

          <!-- Incassato + collection rate bar -->
          <div class="glass-card rounded-2xl overflow-hidden shadow-sm hover-lift animate-in-d2 cursor-default">
            <div class="h-0.5 w-full" style="background: linear-gradient(90deg, #0c8aeb, #0153a2)" />
            <div class="p-5">
              <div class="flex items-center justify-between mb-3">
                <span class="text-[10px] font-bold text-sage-400 uppercase tracking-widest">Incassato</span>
                <div class="w-8 h-8 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #0c8aeb, #0153a2)">
                  <CheckCircle2 class="w-3.5 h-3.5 text-white" />
                </div>
              </div>
              <p class="text-2xl font-bold text-sage-900 tracking-tight tabular-nums">{{ formatCurrency(data.paid_revenue) }}</p>
              <div class="mt-2">
                <div class="flex justify-between items-center mb-1">
                  <span class="text-[11px] text-sage-400">{{ data.paid_invoices }} pagate</span>
                  <span
                    class="text-[11px] font-bold tabular-nums"
                    :class="collectionRate >= 75 ? 'text-sage-600' : 'text-amber-500'"
                  >{{ collectionRate }}%</span>
                </div>
                <div class="h-1 rounded-full bg-sage-100 overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-700"
                    :style="{ width: collectionRate + '%', background: 'linear-gradient(90deg, #0c8aeb, #0153a2)' }"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- Da incassare -->
          <div class="glass-card rounded-2xl overflow-hidden shadow-sm hover-lift animate-in-d3 cursor-default">
            <div class="h-0.5 w-full" style="background: linear-gradient(90deg, #d4a017, #a16207)" />
            <div class="p-5">
              <div class="flex items-center justify-between mb-3">
                <span class="text-[10px] font-bold text-sage-400 uppercase tracking-widest">Da incassare</span>
                <div class="w-8 h-8 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #d4a017, #a16207)">
                  <Clock class="w-3.5 h-3.5 text-white" />
                </div>
              </div>
              <p class="text-2xl font-bold text-sage-900 tracking-tight tabular-nums">{{ formatCurrency(data.unpaid_revenue) }}</p>
              <div class="flex items-center gap-2 mt-1.5">
                <span v-if="issuedUnpaid > 0" class="text-[11px] text-amber-500">{{ issuedUnpaid }} emesse</span>
                <span v-if="issuedUnpaid > 0 && data.draft_invoices > 0" class="text-sage-200 text-xs">·</span>
                <span v-if="data.draft_invoices > 0" class="text-[11px] text-sage-400">{{ data.draft_invoices }} bozze</span>
                <span v-if="issuedUnpaid === 0 && data.draft_invoices === 0" class="text-[11px] text-sage-400">Tutto incassato</span>
              </div>
            </div>
          </div>

          <!-- N. Fatture con status dots -->
          <div class="glass-card rounded-2xl overflow-hidden shadow-sm hover-lift animate-in-d4 cursor-default">
            <div class="h-0.5 w-full" style="background: linear-gradient(90deg, #b88e67, #8a5f42)" />
            <div class="p-5">
              <div class="flex items-center justify-between mb-3">
                <span class="text-[10px] font-bold text-sage-400 uppercase tracking-widest">Fatture</span>
                <div class="w-8 h-8 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #b88e67, #8a5f42)">
                  <FileText class="w-3.5 h-3.5 text-white" />
                </div>
              </div>
              <p class="text-2xl font-bold text-sage-900 tracking-tight tabular-nums">{{ data.total_invoices }}</p>
              <div class="flex items-center gap-3 mt-1.5">
                <span class="flex items-center gap-1 text-[11px] text-sage-500">
                  <span class="w-1.5 h-1.5 rounded-full bg-sage-500 inline-block shrink-0" />
                  {{ data.paid_invoices }} pag.
                </span>
                <span v-if="data.draft_invoices" class="flex items-center gap-1 text-[11px] text-warm-500">
                  <span class="w-1.5 h-1.5 rounded-full bg-warm-400 inline-block shrink-0" />
                  {{ data.draft_invoices }} bozze
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- ── Quick actions strip ─────────────────────────────────────── -->
        <div class="flex gap-2 mb-5 animate-in-d1">
          <button
            type="button"
            class="flex items-center gap-1.5 text-xs font-semibold text-sage-600 hover:text-sage-900 bg-white/60 hover:bg-white border border-sage-200 hover:border-sage-300 px-3.5 py-2 rounded-xl transition-all duration-150 cursor-pointer shadow-sm hover:shadow"
            @click="router.push('/invoices/new')"
          >
            <Plus class="w-3.5 h-3.5" />
            Nuova fattura
          </button>
          <button
            type="button"
            class="flex items-center gap-1.5 text-xs font-semibold text-sage-600 hover:text-sage-900 bg-white/60 hover:bg-white border border-sage-200 hover:border-sage-300 px-3.5 py-2 rounded-xl transition-all duration-150 cursor-pointer shadow-sm hover:shadow"
            @click="router.push('/clients/new')"
          >
            <Users class="w-3.5 h-3.5" />
            Nuovo paziente
          </button>
          <button
            type="button"
            class="flex items-center gap-1.5 text-xs font-semibold text-sage-600 hover:text-sage-900 bg-white/60 hover:bg-white border border-sage-200 hover:border-sage-300 px-3.5 py-2 rounded-xl transition-all duration-150 cursor-pointer shadow-sm hover:shadow"
            @click="router.push('/agenda')"
          >
            <CalendarDays class="w-3.5 h-3.5" />
            Agenda
          </button>
        </div>

        <!-- ── Chart + Tax ─────────────────────────────────────────────── -->
        <div class="grid grid-cols-5 gap-4 mb-5">

          <!-- Monthly bar chart -->
          <div class="col-span-3 glass-card rounded-2xl p-5 shadow-sm animate-in-d2 flex flex-col gap-4">
            <div class="flex items-start justify-between">
              <div>
                <h2 class="text-sm font-semibold text-sage-800">Andamento mensile</h2>
                <p class="text-xs text-sage-400 mt-0.5">Fatturato netto · {{ selectedYear }}</p>
              </div>
              <div v-if="peakMonth" class="shrink-0 ml-4 text-right">
                <p class="text-[10px] uppercase tracking-wider text-sage-400">Mese migliore</p>
                <p class="text-xs font-semibold text-sage-700 mt-0.5">
                  {{ peakMonth.month_name }} &middot; {{ formatCurrency(peakMonth.revenue) }}
                </p>
              </div>
            </div>

            <div class="flex flex-col gap-2">
              <div class="h-48 flex items-end gap-1 relative overflow-visible">
                <!-- Guide lines -->
                <div class="absolute inset-0 flex flex-col justify-between pointer-events-none" style="z-index: -1">
                  <div class="h-px w-full bg-sage-200/40" />
                  <div class="h-px w-full bg-sage-200/35" />
                  <div class="h-px w-full bg-sage-200/30" />
                  <div class="h-px w-full bg-sage-200/25" />
                  <div />
                </div>

                <div
                  v-for="month in data.monthly_revenue"
                  :key="month.month"
                  class="flex-1 h-full flex items-end relative rounded-sm transition-colors duration-150 cursor-default"
                  :class="hoveredMonth === month.month ? 'bg-sage-50/50' : 'bg-transparent'"
                  @mouseenter="hoveredMonth = month.month"
                  @mouseleave="hoveredMonth = null"
                >
                  <!-- Tooltip -->
                  <div
                    v-if="hoveredMonth === month.month && month.revenue > 0"
                    class="absolute left-1/2 -translate-x-1/2 bottom-[calc(100%+8px)] bg-sage-900/95 text-white rounded-lg px-2.5 py-1.5 whitespace-nowrap shadow-xl pointer-events-none z-50"
                    style="font-size: 11px; font-weight: 500"
                  >
                    {{ formatCurrency(month.revenue) }}
                    <span class="text-sage-400 ml-1.5 font-normal" style="font-size: 10px">
                      {{ month.invoice_count }} fatt.
                    </span>
                    <span
                      class="absolute left-1/2 -translate-x-1/2 top-full block"
                      style="width:0;height:0;border-left:5px solid transparent;border-right:5px solid transparent;border-top:5px solid rgba(40,55,43,0.95)"
                    />
                  </div>

                  <!-- Bar -->
                  <div
                    class="w-full rounded-t-md transition-all duration-500"
                    :style="{ height: barHeight(month.revenue), background: barGradient(month) }"
                  />
                </div>
              </div>

              <!-- Month labels -->
              <div class="flex gap-1">
                <div v-for="month in data.monthly_revenue" :key="month.month" class="flex-1 text-center">
                  <span
                    class="text-[10px] transition-colors duration-150"
                    :class="isCurrentMonth(month.month)
                      ? 'text-ocean-500 font-bold'
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

          <!-- Tax estimate -->
          <div class="col-span-2 glass-card rounded-2xl p-5 shadow-sm animate-in-d2 flex flex-col">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-sm font-semibold text-sage-800">Stima Fiscale</h2>
              <span
                class="text-[10px] font-bold px-2.5 py-0.5 rounded-full uppercase tracking-wider"
                :class="isForfettario ? 'bg-sage-100/80 text-sage-600' : 'bg-ocean-50 text-ocean-600'"
              >
                {{ isForfettario ? 'Forfettario' : 'Ordinario' }}
              </span>
            </div>

            <template v-if="taxEstimate">
              <!-- Compensi + proportion bar -->
              <div class="mb-4">
                <p class="text-[10px] text-sage-400 uppercase tracking-widest mb-0.5">Compensi annui</p>
                <p class="text-xl font-bold text-sage-900 tracking-tight tabular-nums">{{ formatCurrency(data.total_net_revenue) }}</p>
                <div class="mt-2">
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
              </div>

              <!-- Breakdown rows -->
              <div class="space-y-1.5 flex-1">
                <div class="flex justify-between text-xs">
                  <span class="text-sage-500">Redd. imponibile <span class="text-sage-300">({{ configStore.config?.coefficient }}%)</span></span>
                  <span class="text-sage-700 font-medium tabular-nums">{{ formatCurrency(taxEstimate.taxableIncome) }}</span>
                </div>
                <div class="flex justify-between text-xs">
                  <span class="text-sage-500">Contributi prev.</span>
                  <span class="text-amber-600 font-medium tabular-nums">&minus;{{ formatCurrency(taxEstimate.inpsContribution) }}</span>
                </div>

                <template v-if="taxEstimate.type === 'forfettario'">
                  <div class="flex justify-between text-xs">
                    <span class="text-sage-500">Imp. sostitutiva <span class="text-sage-300">({{ taxEstimate.substituteTaxRate }}%)</span></span>
                    <span class="text-amber-600 font-medium tabular-nums">&minus;{{ formatCurrency(taxEstimate.substituteTax) }}</span>
                  </div>
                </template>
                <template v-else>
                  <div class="flex justify-between text-xs">
                    <span class="text-sage-500">IRPEF</span>
                    <span class="text-amber-600 font-medium tabular-nums">&minus;{{ formatCurrency(taxEstimate.irpef) }}</span>
                  </div>
                  <div class="flex justify-between text-xs">
                    <span class="text-sage-500">Addizionali</span>
                    <span class="text-amber-600 font-medium tabular-nums">&minus;{{ formatCurrency(taxEstimate.addizionaleRegionale + taxEstimate.addizionaleComunale) }}</span>
                  </div>
                </template>
              </div>

              <!-- Net result gradient box -->
              <div class="mt-3 rounded-xl px-4 py-3" style="background: linear-gradient(135deg, #3a513e, #5d8062)">
                <p class="text-[10px] font-bold text-white/60 uppercase tracking-widest mb-0.5">Netto stimato</p>
                <p class="text-xl font-bold text-white tracking-tight tabular-nums">{{ formatCurrency(taxEstimate.netIncome) }}</p>
              </div>

              <!-- Tax rate toggle (forfettario only) -->
              <div v-if="isForfettario" class="flex items-center gap-1.5 mt-3 pt-3 border-t border-sage-100/50">
                <span class="text-[10px] text-sage-400 mr-1">Aliquota</span>
                <button
                  type="button"
                  class="text-[10px] px-2.5 py-1 rounded-lg transition-all cursor-pointer font-bold"
                  :class="firstFiveYears ? 'bg-sage-700 text-white' : 'bg-sage-50 text-sage-400 hover:text-sage-600'"
                  @click="firstFiveYears = true"
                >5%</button>
                <button
                  type="button"
                  class="text-[10px] px-2.5 py-1 rounded-lg transition-all cursor-pointer font-bold"
                  :class="!firstFiveYears ? 'bg-sage-700 text-white' : 'bg-sage-50 text-sage-400 hover:text-sage-600'"
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
        <div class="glass-card rounded-2xl shadow-sm animate-in-d3 overflow-hidden">
          <div class="px-6 py-4 flex items-center justify-between border-b border-sage-100/50">
            <div>
              <h2 class="text-sm font-semibold text-sage-800">Fatture recenti</h2>
              <p class="text-xs text-sage-400 mt-0.5">Ultime attività dell'anno</p>
            </div>
            <button
              type="button"
              class="group flex items-center gap-1 text-xs font-semibold text-sage-500 hover:text-sage-800 transition-colors cursor-pointer"
              @click="router.push('/invoices')"
            >
              Vedi tutte
              <ArrowRight class="w-3.5 h-3.5 transition-transform group-hover:translate-x-0.5" />
            </button>
          </div>

          <div v-if="data.recent_invoices.length === 0" class="px-6 py-12 text-center">
            <div class="w-12 h-12 rounded-2xl bg-sage-50 flex items-center justify-center mx-auto mb-3">
              <FileText class="w-6 h-6 text-sage-200" />
            </div>
            <p class="text-sm font-semibold text-sage-500">Nessuna fattura</p>
            <p class="text-xs text-sage-400 mt-1">Non ci sono fatture per il {{ selectedYear }}.</p>
          </div>

          <table v-else class="w-full text-sm">
            <thead>
              <tr class="border-b border-sage-100/60 bg-sage-50/30">
                <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Numero</th>
                <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Cliente</th>
                <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Data</th>
                <th class="px-5 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Totale</th>
                <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Stato</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-sage-50">
              <tr
                v-for="(invoice, idx) in data.recent_invoices"
                :key="invoice.id"
                :style="{ '--i': Math.min(idx, 8) }"
                class="invoice-row hover:bg-sage-50/60 cursor-pointer transition-colors duration-150"
                @click="router.push(`/invoices/${invoice.id}`)"
              >
                <td class="px-5 py-3.5">
                  <span class="font-mono text-xs font-semibold text-sage-700 bg-sage-50 border border-sage-100 px-2 py-1 rounded-lg">
                    {{ invoice.invoice_number }}
                  </span>
                </td>
                <td class="px-5 py-3.5">
                  <div class="flex items-center gap-2.5">
                    <div
                      class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0 text-white text-[10px] font-bold shadow-sm"
                      :style="{ background: clientAvatarGradient(invoice.id) }"
                    >
                      {{ clientInitials(invoice.client_name) }}
                    </div>
                    <span class="text-sm text-sage-700 font-medium truncate max-w-[160px]">{{ invoice.client_name }}</span>
                  </div>
                </td>
                <td class="px-5 py-3.5 text-xs text-sage-400">{{ formatDate(invoice.issue_date) }}</td>
                <td class="px-5 py-3.5 text-right font-semibold text-sage-800 tabular-nums">{{ formatCurrency(invoice.total_due) }}</td>
                <td class="px-5 py-3.5">
                  <StatusBadge :status="invoice.status" type="invoice" />
                </td>
              </tr>
            </tbody>
          </table>
        </div>

      </template>

      <!-- ── No data ─────────────────────────────────────────────────────── -->
      <div v-else class="flex items-center justify-center h-64">
        <div class="text-center">
          <p class="text-sm text-sage-400">Nessun dato disponibile.</p>
          <button
            type="button"
            class="mt-3 text-sm font-medium text-sage-600 hover:text-sage-800 transition-colors cursor-pointer"
            @click="loadDashboard"
          >
            Ricarica
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
.invoice-row {
  animation: row-in 0.3s ease both;
  animation-delay: calc(var(--i, 0) * 30ms);
}

@keyframes row-in {
  from { opacity: 0; transform: translateX(-5px); }
  to   { opacity: 1; transform: translateX(0); }
}

@media (prefers-reduced-motion: reduce) {
  .invoice-row { animation: none; }
}
</style>
