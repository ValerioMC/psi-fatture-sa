<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { TrendingUp, Euro, Clock, FileText, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { getDashboard } from '@/api'
import type { DashboardData } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { formatCurrency, formatDate } from '@/utils/format'

const router = useRouter()

const currentYear = new Date().getFullYear()
const selectedYear = ref(currentYear)
const data = ref<DashboardData | null>(null)
const loading = ref(false)

async function loadDashboard() {
  loading.value = true
  try {
    data.value = await getDashboard(selectedYear.value)
  } finally {
    loading.value = false
  }
}

function prevYear() {
  selectedYear.value--
  loadDashboard()
}

function nextYear() {
  selectedYear.value++
  loadDashboard()
}

const maxRevenue = computed(() => {
  if (!data.value) return 1
  const max = Math.max(...data.value.monthly_revenue.map((m) => m.revenue), 1)
  return max
})

function barHeight(revenue: number): string {
  const pct = (revenue / maxRevenue.value) * 100
  return `${Math.max(pct, 2)}%`
}

onMounted(loadDashboard)
</script>

<template>
  <div class="p-8">
    <PageHeader title="Dashboard">
      <div class="flex items-center gap-2 bg-white border border-gray-200 rounded-lg px-3 py-1.5">
        <button
          type="button"
          class="text-gray-500 hover:text-gray-800 transition-colors"
          @click="prevYear"
        >
          <ChevronLeft class="w-4 h-4" />
        </button>
        <span class="text-sm font-medium text-gray-900 min-w-[3rem] text-center">
          {{ selectedYear }}
        </span>
        <button
          type="button"
          class="text-gray-500 hover:text-gray-800 transition-colors"
          @click="nextYear"
        >
          <ChevronRight class="w-4 h-4" />
        </button>
      </div>
    </PageHeader>

    <div v-if="loading" class="flex items-center justify-center h-64 text-gray-400 text-sm">
      Caricamento...
    </div>

    <template v-else-if="data">
      <!-- Stat cards -->
      <div class="grid grid-cols-4 gap-4 mb-6">
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-5">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm text-gray-500">Fatturato totale</span>
            <div class="w-8 h-8 rounded-lg bg-blue-50 flex items-center justify-center">
              <TrendingUp class="w-4 h-4 text-blue-600" />
            </div>
          </div>
          <p class="text-2xl font-bold text-gray-900">{{ formatCurrency(data.total_revenue) }}</p>
          <p class="text-xs text-gray-400 mt-1">{{ data.total_invoices }} fatture totali</p>
        </div>

        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-5">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm text-gray-500">Incassato</span>
            <div class="w-8 h-8 rounded-lg bg-green-50 flex items-center justify-center">
              <Euro class="w-4 h-4 text-green-600" />
            </div>
          </div>
          <p class="text-2xl font-bold text-gray-900">{{ formatCurrency(data.paid_revenue) }}</p>
          <p class="text-xs text-gray-400 mt-1">{{ data.paid_invoices }} fatture pagate</p>
        </div>

        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-5">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm text-gray-500">Da incassare</span>
            <div class="w-8 h-8 rounded-lg bg-amber-50 flex items-center justify-center">
              <Clock class="w-4 h-4 text-amber-600" />
            </div>
          </div>
          <p class="text-2xl font-bold text-gray-900">{{ formatCurrency(data.unpaid_revenue) }}</p>
          <p class="text-xs text-gray-400 mt-1">{{ data.draft_invoices }} bozze</p>
        </div>

        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-5">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm text-gray-500">N. fatture</span>
            <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center">
              <FileText class="w-4 h-4 text-purple-600" />
            </div>
          </div>
          <p class="text-2xl font-bold text-gray-900">{{ data.total_invoices }}</p>
          <p class="text-xs text-gray-400 mt-1">nell'anno {{ selectedYear }}</p>
        </div>
      </div>

      <!-- Monthly bar chart -->
      <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 mb-6">
        <h2 class="text-sm font-semibold text-gray-900 mb-4">Andamento mensile</h2>
        <div class="flex items-end gap-2 h-40">
          <div
            v-for="month in data.monthly_revenue"
            :key="month.month"
            class="flex-1 flex flex-col items-center gap-1"
          >
            <span class="text-xs text-gray-400 whitespace-nowrap">
              {{ month.revenue > 0 ? formatCurrency(month.revenue) : '' }}
            </span>
            <div class="w-full flex items-end justify-center" style="height: 100px">
              <div
                class="w-full rounded-t-md transition-all"
                :class="month.revenue > 0 ? 'bg-blue-500' : 'bg-gray-100'"
                :style="{ height: month.revenue > 0 ? barHeight(month.revenue) : '4px' }"
                :title="`${month.month_name}: ${formatCurrency(month.revenue)}`"
              />
            </div>
            <span class="text-xs text-gray-500 truncate w-full text-center">
              {{ month.month_name.slice(0, 3) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Recent invoices -->
      <div class="bg-white rounded-xl border border-gray-100 shadow-sm">
        <div class="px-6 py-4 border-b border-gray-100 flex items-center justify-between">
          <h2 class="text-sm font-semibold text-gray-900">Fatture recenti</h2>
          <button
            type="button"
            class="text-xs text-blue-600 hover:text-blue-700 font-medium"
            @click="router.push('/invoices')"
          >
            Vedi tutte
          </button>
        </div>
        <div v-if="data.recent_invoices.length === 0" class="px-6 py-8 text-center text-sm text-gray-400">
          Nessuna fattura emessa.
        </div>
        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-gray-100">
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Numero
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Cliente
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Data
              </th>
              <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                Totale
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Stato
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-50">
            <tr
              v-for="invoice in data.recent_invoices"
              :key="invoice.id"
              class="hover:bg-gray-50 cursor-pointer"
              @click="router.push(`/invoices/${invoice.id}`)"
            >
              <td class="px-6 py-3 font-medium text-gray-900">{{ invoice.invoice_number }}</td>
              <td class="px-6 py-3 text-gray-600">{{ invoice.client_name }}</td>
              <td class="px-6 py-3 text-gray-500">{{ formatDate(invoice.issue_date) }}</td>
              <td class="px-6 py-3 text-right font-medium text-gray-900">
                {{ formatCurrency(invoice.total_due) }}
              </td>
              <td class="px-6 py-3">
                <StatusBadge :status="invoice.status" type="invoice" />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>
