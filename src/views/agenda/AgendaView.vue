<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { ChevronLeft, ChevronRight, Plus, CalendarDays, CheckCircle2, Clock } from 'lucide-vue-next'
import { listAppointments } from '@/api'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import type { Appointment } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppointmentModal from './AppointmentModal.vue'

const clientsStore = useClientsStore()
const servicesStore = useServicesStore()

const today = new Date()
const todayStr = today.toISOString().slice(0, 10)
const viewYear = ref(today.getFullYear())
const viewMonth = ref(today.getMonth())
const appointments = ref<Appointment[]>([])
const loading = ref(false)
const selectedDate = ref(todayStr)

const showModal = ref(false)
const modalDate = ref<string | undefined>(undefined)
const editingAppointment = ref<Appointment | null>(null)
const dayPanelRef = ref<HTMLElement | null>(null)

const MONTH_NAMES = [
  'Gennaio', 'Febbraio', 'Marzo', 'Aprile', 'Maggio', 'Giugno',
  'Luglio', 'Agosto', 'Settembre', 'Ottobre', 'Novembre', 'Dicembre',
]
const DAY_NAMES = ['Lun', 'Mar', 'Mer', 'Gio', 'Ven', 'Sab', 'Dom']

function padDate(y: number, m: number, d: number): string {
  return `${y}-${String(m + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`
}

const calendarDays = computed(() => {
  const year = viewYear.value
  const month = viewMonth.value
  const firstDay = new Date(year, month, 1)
  const lastDay = new Date(year, month + 1, 0)
  const startDow = (firstDay.getDay() + 6) % 7
  const days: Array<{ date: string; day: number; currentMonth: boolean }> = []

  for (let i = startDow - 1; i >= 0; i--) {
    const d = new Date(year, month, -i)
    days.push({ date: padDate(d.getFullYear(), d.getMonth(), d.getDate()), day: d.getDate(), currentMonth: false })
  }
  for (let d = 1; d <= lastDay.getDate(); d++) {
    days.push({ date: padDate(year, month, d), day: d, currentMonth: true })
  }
  const remainder = days.length % 7
  if (remainder !== 0) {
    for (let i = 1; i <= 7 - remainder; i++) {
      const d = new Date(year, month + 1, i)
      days.push({ date: padDate(d.getFullYear(), d.getMonth(), d.getDate()), day: d.getDate(), currentMonth: false })
    }
  }
  return days
})

const appointmentsByDate = computed(() => {
  const map: Record<string, Appointment[]> = {}
  for (const appt of appointments.value) {
    if (!map[appt.date]) map[appt.date] = []
    map[appt.date].push(appt)
  }
  return map
})

const selectedDayAppointments = computed(() =>
  [...(appointmentsByDate.value[selectedDate.value] ?? [])].sort((a, b) =>
    a.start_time.localeCompare(b.start_time),
  ),
)

const monthStats = computed(() => {
  const all = appointments.value
  return {
    total: all.length,
    completed: all.filter(a => a.status === 'completed').length,
    todayCount: (appointmentsByDate.value[todayStr] ?? []).length,
  }
})

const selectedDayStats = computed(() => {
  const appts = selectedDayAppointments.value
  return {
    total: appts.length,
    completed: appts.filter(a => a.status === 'completed').length,
    scheduled: appts.filter(a => a.status === 'scheduled').length,
  }
})

const selectedDayLabel = computed(() =>
  new Date(selectedDate.value + 'T00:00:00').toLocaleDateString('it-IT', {
    weekday: 'long',
    day: 'numeric',
    month: 'long',
  }),
)

function prevMonth() {
  if (viewMonth.value === 0) { viewMonth.value = 11; viewYear.value-- }
  else viewMonth.value--
}

function nextMonth() {
  if (viewMonth.value === 11) { viewMonth.value = 0; viewYear.value++ }
  else viewMonth.value++
}

async function loadAppointments() {
  loading.value = true
  const year = viewYear.value
  const month = viewMonth.value
  const dateFrom = padDate(year, month, 1)
  const dateTo = padDate(year, month, new Date(year, month + 1, 0).getDate())
  try {
    appointments.value = await listAppointments(dateFrom, dateTo)
  } finally {
    loading.value = false
  }
}

function selectDate(date: string) {
  selectedDate.value = date
  nextTick(() => {
    dayPanelRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  })
}

function openCreate(date: string) {
  editingAppointment.value = null
  modalDate.value = date
  showModal.value = true
}

function openEdit(appt: Appointment) {
  editingAppointment.value = appt
  modalDate.value = appt.date
  showModal.value = true
}

function onModalSaved() {
  loadAppointments()
}

function statusDotClass(status: string): string {
  const map: Record<string, string> = {
    scheduled: 'bg-ocean-400 border-ocean-300',
    completed: 'bg-emerald-500 border-emerald-300',
    cancelled: 'bg-warm-300 border-warm-200',
  }
  return map[status] ?? 'bg-sage-300'
}

function statusPillClass(status: string): string {
  const map: Record<string, string> = {
    scheduled: 'bg-ocean-50 text-ocean-700 border border-ocean-100',
    completed: 'bg-emerald-50 text-emerald-700 border border-emerald-100',
    cancelled: 'bg-warm-50 text-warm-500 border border-warm-100 line-through',
  }
  return map[status] ?? 'bg-sage-50 text-sage-500'
}

function statusAccentClass(status: string): string {
  const map: Record<string, string> = {
    scheduled: 'bg-ocean-400',
    completed: 'bg-emerald-500',
    cancelled: 'bg-warm-300',
  }
  return map[status] ?? 'bg-sage-300'
}

function statusBorderClass(status: string): string {
  const map: Record<string, string> = {
    scheduled: 'border-ocean-100',
    completed: 'border-emerald-100',
    cancelled: 'border-warm-100',
  }
  return map[status] ?? 'border-sage-100'
}

function statusLabel(status: string): string {
  const map: Record<string, string> = {
    scheduled: 'Programmato',
    completed: 'Completato',
    cancelled: 'Annullato',
  }
  return map[status] ?? status
}

function apptDuration(appt: Appointment): string {
  const [sh, sm] = appt.start_time.split(':').map(Number)
  const [eh, em] = appt.end_time.split(':').map(Number)
  const mins = (eh * 60 + em) - (sh * 60 + sm)
  return mins > 0 ? `${mins} min` : ''
}

function calendarCellAppts(date: string): Appointment[] {
  return appointmentsByDate.value[date] ?? []
}

function cellDotColor(status: string): string {
  const map: Record<string, string> = {
    scheduled: 'bg-ocean-400',
    completed: 'bg-emerald-500',
    cancelled: 'bg-warm-300',
  }
  return map[status] ?? 'bg-sage-300'
}

const isToday = (dateStr: string) => dateStr === todayStr

watch([viewYear, viewMonth], loadAppointments)
onMounted(async () => {
  await Promise.all([
    clientsStore.fetchClients(),
    servicesStore.fetchServices(false),
    loadAppointments(),
  ])
})
</script>

<template>
  <div class="p-8">
    <div class="max-w-6xl mx-auto">
      <PageHeader title="Agenda" subtitle="Gestisci i tuoi appuntamenti.">
        <button
          type="button"
          class="group relative overflow-hidden text-white font-semibold px-4 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 cursor-pointer focus:outline-none"
          style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
          @click="openCreate(selectedDate)"
        >
          <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
          <Plus class="w-4 h-4 relative z-10" />
          <span class="relative z-10">Nuovo Appuntamento</span>
        </button>
      </PageHeader>

      <!-- Stats strip -->
      <div class="grid grid-cols-3 gap-4 mb-6 animate-in">
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #4f46e5, #4338ca)">
            <CalendarDays class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Questo mese</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ monthStats.total }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #6366f1, #4f46e5)">
            <Clock class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Oggi</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ monthStats.todayCount }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #059669, #047857)">
            <CheckCircle2 class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Completati</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ monthStats.completed }}</p>
          </div>
        </div>
      </div>

      <!-- Calendar (full width) -->
      <div class="glass-card rounded-2xl shadow-sm overflow-hidden animate-in-d1">
        <!-- Month nav -->
        <div class="flex items-center justify-between px-5 py-4 border-b border-sage-100/60">
          <button
            type="button"
            class="p-1.5 text-sage-500 hover:bg-sage-100 rounded-lg transition-colors cursor-pointer"
            @click="prevMonth"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <h2 class="text-sm font-semibold text-sage-900">
            {{ MONTH_NAMES[viewMonth] }} {{ viewYear }}
          </h2>
          <button
            type="button"
            class="p-1.5 text-sage-500 hover:bg-sage-100 rounded-lg transition-colors cursor-pointer"
            @click="nextMonth"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        </div>

        <!-- Day name headers -->
        <div class="grid grid-cols-7 border-b border-sage-100/60">
          <div
            v-for="dayName in DAY_NAMES"
            :key="dayName"
            class="text-center text-[10px] font-semibold text-sage-400 uppercase tracking-wider py-2"
          >
            {{ dayName }}
          </div>
        </div>

        <!-- Loading overlay -->
        <div v-if="loading" class="flex items-center justify-center py-16">
          <div class="w-6 h-6 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        </div>

        <!-- Calendar grid -->
        <div v-else class="grid grid-cols-7 divide-x divide-y divide-sage-100/40">
          <div
            v-for="cell in calendarDays"
            :key="cell.date"
            class="min-h-[80px] p-2 cursor-pointer transition-all duration-200 active:scale-95 relative"
            :class="[
              cell.date === selectedDate
                ? 'bg-sage-50/80 ring-2 ring-inset ring-sage-400/50'
                : 'hover:bg-sage-50/40',
              !cell.currentMonth ? 'opacity-35' : '',
            ]"
            @click="selectDate(cell.date)"
          >
            <!-- Day number -->
            <div class="flex items-start justify-between mb-1.5">
              <span
                class="text-xs font-semibold w-6 h-6 flex items-center justify-center rounded-full leading-none"
                :class="isToday(cell.date)
                  ? 'bg-gradient-to-br from-sage-600 to-ocean-500 text-white'
                  : cell.date === selectedDate
                    ? 'text-sage-800'
                    : 'text-sage-600'"
              >
                {{ cell.day }}
              </span>
              <!-- Count badge -->
              <span
                v-if="calendarCellAppts(cell.date).length > 0"
                class="text-[9px] font-bold px-1.5 py-0.5 rounded-full"
                :class="calendarCellAppts(cell.date).length >= 6
                  ? 'bg-ocean-100 text-ocean-700'
                  : 'bg-sage-100 text-sage-600'"
              >
                {{ calendarCellAppts(cell.date).length }}
              </span>
            </div>

            <!-- Status dot row -->
            <div v-if="calendarCellAppts(cell.date).length > 0" class="flex flex-wrap gap-0.5 mb-1">
              <div
                v-for="appt in calendarCellAppts(cell.date).slice(0, 6)"
                :key="appt.id"
                class="w-1.5 h-1.5 rounded-full"
                :class="cellDotColor(appt.status)"
              />
              <div
                v-if="calendarCellAppts(cell.date).length > 6"
                class="text-[8px] text-sage-400 leading-none mt-px"
              >
                +{{ calendarCellAppts(cell.date).length - 6 }}
              </div>
            </div>

            <!-- First appointment preview -->
            <div
              v-if="calendarCellAppts(cell.date).length > 0"
              class="text-[9px] text-sage-500 truncate leading-tight"
            >
              {{ calendarCellAppts(cell.date)[0].start_time.slice(0, 5) }}
              {{ calendarCellAppts(cell.date)[0].client_name.split(' ')[0] }}
            </div>
          </div>
        </div>
      </div>

      <!-- Day panel (below calendar, full width) -->
      <div ref="dayPanelRef" class="glass-card rounded-2xl shadow-sm overflow-hidden mt-5 animate-in-d2 scroll-mt-6">
        <!-- Day panel header -->
        <div class="px-5 py-4 border-b border-sage-100/60">
          <div class="flex items-center justify-between gap-4">
            <Transition name="header-fade" mode="out-in">
              <div :key="selectedDate" class="flex items-center gap-4">
                <div>
                  <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Giornata</p>
                  <h3 class="text-sm font-semibold text-sage-900 capitalize leading-tight">
                    {{ selectedDayLabel }}
                  </h3>
                </div>
                <!-- Day stats pills -->
                <div v-if="selectedDayStats.total > 0" class="flex gap-2">
                  <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-sage-100 text-sage-600">
                    {{ selectedDayStats.total }} appunt.
                  </span>
                  <span v-if="selectedDayStats.completed > 0" class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-sage-50 text-sage-500">
                    {{ selectedDayStats.completed }} completati
                  </span>
                  <span v-if="selectedDayStats.scheduled > 0" class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-ocean-50 text-ocean-600">
                    {{ selectedDayStats.scheduled }} da fare
                  </span>
                </div>
              </div>
            </Transition>
            <button
              type="button"
              class="shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-sage-50 hover:bg-sage-100 text-sage-600 hover:text-sage-800 text-xs font-medium transition-colors cursor-pointer"
              title="Aggiungi appuntamento"
              @click="openCreate(selectedDate)"
            >
              <Plus class="w-3.5 h-3.5" />
              Aggiungi
            </button>
          </div>
        </div>

        <!-- Body: transitions on date change -->
        <Transition name="day-content" mode="out-in">
          <div :key="selectedDate">

        <!-- Empty state -->
        <div v-if="selectedDayAppointments.length === 0" class="flex flex-col items-center justify-center py-14 text-center px-5">
          <div class="w-12 h-12 rounded-2xl bg-sage-50 flex items-center justify-center mb-3">
            <CalendarDays class="w-6 h-6 text-sage-300" />
          </div>
          <p class="text-sm font-medium text-sage-500">Nessun appuntamento</p>
          <p class="text-xs text-sage-400 mt-1">Clicca Aggiungi per inserirne uno.</p>
        </div>

        <!-- Appointments grid -->
        <div v-else class="p-4 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          <div
            v-for="(appt, idx) in selectedDayAppointments"
            :key="appt.id"
            :style="{ '--i': idx }"
            class="appt-card rounded-xl border bg-white/80 shadow-sm overflow-hidden cursor-pointer transition-all hover:shadow-md hover:bg-white group"
            :class="[
              appt.status === 'cancelled' ? 'opacity-50' : '',
              statusBorderClass(appt.status),
            ]"
            @click="openEdit(appt)"
          >
            <div class="flex h-full">
              <!-- Left accent strip -->
              <div class="w-[3px] shrink-0 self-stretch" :class="statusAccentClass(appt.status)" />

              <div class="flex-1 min-w-0 px-3 py-3">
                <!-- Time + status badge -->
                <div class="flex items-start justify-between gap-2 mb-2">
                  <div class="flex items-center gap-1.5">
                    <div
                      class="w-2 h-2 rounded-full shrink-0 border-2 transition-transform group-hover:scale-125"
                      :class="statusDotClass(appt.status)"
                    />
                    <span class="text-[11px] font-mono font-semibold text-sage-600 tabular-nums">
                      {{ appt.start_time.slice(0, 5) }}
                    </span>
                  </div>
                  <span
                    class="shrink-0 text-[9px] font-semibold px-1.5 py-0.5 rounded-full"
                    :class="statusPillClass(appt.status)"
                  >
                    {{ statusLabel(appt.status) }}
                  </span>
                </div>

                <!-- Client name -->
                <p class="text-sm font-semibold text-sage-900 truncate leading-tight">{{ appt.client_name }}</p>

                <!-- Service name -->
                <p v-if="appt.service_name" class="text-xs text-sage-500 truncate mt-0.5 leading-tight">{{ appt.service_name }}</p>

                <!-- Time range + duration -->
                <div class="flex items-center gap-1.5 mt-2">
                  <Clock class="w-3 h-3 text-sage-300 shrink-0" />
                  <span class="text-[10px] text-sage-400 tabular-nums">{{ appt.start_time.slice(0, 5) }} – {{ appt.end_time.slice(0, 5) }}</span>
                  <span v-if="apptDuration(appt)" class="text-[10px] text-sage-300">·</span>
                  <span v-if="apptDuration(appt)" class="text-[10px] text-sage-400">{{ apptDuration(appt) }}</span>
                </div>

                <!-- Notes -->
                <p v-if="appt.notes" class="text-[10px] text-sage-400 mt-1.5 italic truncate">{{ appt.notes }}</p>
              </div>
            </div>
          </div>
        </div>

          </div>
        </Transition>
      </div>

    </div>
  </div>

  <AppointmentModal
    :open="showModal"
    :date="modalDate"
    :appointment="editingAppointment"
    @close="showModal = false"
    @saved="onModalSaved"
  />
</template>

<style scoped>
/* Header date label: fade + tiny slide */
.header-fade-enter-active { transition: opacity 0.18s ease, transform 0.18s ease; }
.header-fade-leave-active  { transition: opacity 0.1s ease,  transform 0.1s ease;  }
.header-fade-enter-from    { opacity: 0; transform: translateY(4px);  }
.header-fade-leave-to      { opacity: 0; transform: translateY(-4px); }

/* Day panel body: fade + slide up */
.day-content-enter-active { transition: opacity 0.22s ease, transform 0.22s ease; }
.day-content-leave-active { transition: opacity 0.12s ease, transform 0.12s ease; }
.day-content-enter-from   { opacity: 0; transform: translateY(10px); }
.day-content-leave-to     { opacity: 0; transform: translateY(-6px); }

/* Appointment cards: staggered slide-up on enter */
.appt-card {
  animation: appt-in 0.32s ease both;
  animation-delay: calc(var(--i, 0) * 45ms);
}

@keyframes appt-in {
  from { opacity: 0; transform: translateY(12px); }
  to   { opacity: 1; transform: translateY(0);    }
}

@media (prefers-reduced-motion: reduce) {
  .header-fade-enter-active,
  .header-fade-leave-active,
  .day-content-enter-active,
  .day-content-leave-active { transition: opacity 0.1s ease; }
  .header-fade-enter-from,
  .header-fade-leave-to,
  .day-content-enter-from,
  .day-content-leave-to { transform: none; }
  .appt-card { animation: none; }
}
</style>
