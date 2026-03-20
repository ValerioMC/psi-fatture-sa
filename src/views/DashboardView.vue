<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { TrendingUp, Euro, Clock, FileText, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { getDashboard } from '@/api'
import type { DashboardData } from '@/types'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { formatCurrency, formatDate } from '@/utils/format'

const router = useRouter()
const currentYear = new Date().getFullYear()
const selectedYear = ref(currentYear)
const data = ref<DashboardData | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

async function loadDashboard() {
  loading.value = true
  error.value = null
  try {
    data.value = await getDashboard(selectedYear.value)
  } catch (e) {
    error.value = String(e)
    data.value = null
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
  return `${Math.max((revenue / maxRevenue.value) * 100, 2)}%`
}

onMounted(loadDashboard)
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">
    <!-- Header -->
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

    <div v-if="loading" class="flex items-center justify-center h-64">
      <div class="flex flex-col items-center gap-3">
        <div
          class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin"
        />
        <p class="text-sm text-sage-400">Caricamento...</p>
      </div>
    </div>

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
      <!-- KPI cards -->
      <div class="grid grid-cols-4 gap-4 mb-6">
        <!-- Fatturato totale -->
        <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d1">
          <div class="flex items-center justify-between mb-4">
            <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Fatturato totale</span>
            <div
              class="w-9 h-9 rounded-xl flex items-center justify-center"
              style="background: linear-gradient(135deg, #5d8062, #48654c)"
            >
              <TrendingUp class="w-4 h-4 text-white" />
            </div>
          </div>
          <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ formatCurrency(data.total_revenue) }}</p>
          <p class="text-xs text-sage-400 mt-1.5">{{ data.total_invoices }} fatture totali</p>
        </div>

        <!-- Incassato -->
        <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d2">
          <div class="flex items-center justify-between mb-4">
            <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Incassato</span>
            <div
              class="w-9 h-9 rounded-xl flex items-center justify-center"
              style="background: linear-gradient(135deg, #0c8aeb, #0153a2)"
            >
              <Euro class="w-4 h-4 text-white" />
            </div>
          </div>
          <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ formatCurrency(data.paid_revenue) }}</p>
          <p class="text-xs text-sage-400 mt-1.5">{{ data.paid_invoices }} fatture pagate</p>
        </div>

        <!-- Da incassare -->
        <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d3">
          <div class="flex items-center justify-between mb-4">
            <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Da incassare</span>
            <div
              class="w-9 h-9 rounded-xl flex items-center justify-center"
              style="background: linear-gradient(135deg, #d4a017, #a16207)"
            >
              <Clock class="w-4 h-4 text-white" />
            </div>
          </div>
          <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ formatCurrency(data.unpaid_revenue) }}</p>
          <p class="text-xs text-sage-400 mt-1.5">{{ data.draft_invoices }} bozze</p>
        </div>

        <!-- N. fatture -->
        <div class="glass-card rounded-2xl p-5 shadow-sm hover-lift animate-in-d4">
          <div class="flex items-center justify-between mb-4">
            <span class="text-xs font-semibold text-sage-500 uppercase tracking-wider">N. Fatture</span>
            <div
              class="w-9 h-9 rounded-xl flex items-center justify-center"
              style="background: linear-gradient(135deg, #b88e67, #8a5f42)"
            >
              <FileText class="w-4 h-4 text-white" />
            </div>
          </div>
          <p class="text-2xl font-bold text-sage-900 tracking-tight">{{ data.total_invoices }}</p>
          <p class="text-xs text-sage-400 mt-1.5">nell'anno {{ selectedYear }}</p>
        </div>
      </div>

      <!-- Bar chart -->
      <div class="glass-card rounded-2xl p-6 shadow-sm mb-6 animate-in-d2">
        <h2 class="text-sm font-semibold text-sage-800 mb-5">Andamento mensile</h2>
        <div class="flex items-end gap-1.5" style="height: 140px">
          <div
            v-for="month in data.monthly_revenue"
            :key="month.month"
            class="flex-1 flex flex-col items-center gap-1.5"
          >
            <span
              class="text-[10px] text-sage-400 whitespace-nowrap transition-opacity duration-300"
              :style="{ opacity: month.revenue > 0 ? 1 : 0 }"
            >
              {{ month.revenue > 0 ? formatCurrency(month.revenue) : '' }}
            </span>
            <div class="w-full flex items-end justify-center" style="height: 90px">
              <div
                class="w-full rounded-t-lg transition-all duration-500"
                :style="{
                  height: month.revenue > 0 ? barHeight(month.revenue) : '3px',
                  background: month.revenue > 0
                    ? 'linear-gradient(to top, #48654c, #5d8062, #0c8aeb)'
                    : 'rgba(163,186,163,0.25)',
                  minHeight: '3px',
                }"
                :title="`${month.month_name}: ${formatCurrency(month.revenue)}`"
              />
            </div>
            <span class="text-[10px] text-sage-400 truncate w-full text-center">
              {{ month.month_name.slice(0, 3) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Recent invoices -->
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
