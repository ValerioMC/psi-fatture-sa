<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { ChevronLeft, ChevronRight, Plus } from 'lucide-vue-next'
import { listAppointments } from '@/api'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import type { Appointment } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import AppointmentModal from './AppointmentModal.vue'

const clientsStore = useClientsStore()
const servicesStore = useServicesStore()

const today = new Date()
const viewYear = ref(today.getFullYear())
const viewMonth = ref(today.getMonth()) // 0-indexed
const appointments = ref<Appointment[]>([])
const loading = ref(false)
const selectedDate = ref(today.toISOString().slice(0, 10))

const showModal = ref(false)
const modalDate = ref<string | undefined>(undefined)
const editingAppointment = ref<Appointment | null>(null)

const MONTH_NAMES = [
  'Gennaio', 'Febbraio', 'Marzo', 'Aprile', 'Maggio', 'Giugno',
  'Luglio', 'Agosto', 'Settembre', 'Ottobre', 'Novembre', 'Dicembre',
]

const DAY_NAMES = ['Lun', 'Mar', 'Mer', 'Gio', 'Ven', 'Sab', 'Dom']

/** Pad ISO date string */
function padDate(y: number, m: number, d: number): string {
  return `${y}-${String(m + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`
}

/** Build the calendar grid for the current month view. */
const calendarDays = computed(() => {
  const year = viewYear.value
  const month = viewMonth.value
  const firstDay = new Date(year, month, 1)
  const lastDay = new Date(year, month + 1, 0)

  // Monday-first: convert Sunday=0 → 6, Mon=1 → 0, etc.
  const startDow = (firstDay.getDay() + 6) % 7
  const days: Array<{ date: string; day: number; currentMonth: boolean }> = []

  // Prepend trailing days from previous month
  for (let i = startDow - 1; i >= 0; i--) {
    const d = new Date(year, month, -i)
    days.push({ date: padDate(d.getFullYear(), d.getMonth(), d.getDate()), day: d.getDate(), currentMonth: false })
  }

  // Current month days
  for (let d = 1; d <= lastDay.getDate(); d++) {
    days.push({ date: padDate(year, month, d), day: d, currentMonth: true })
  }

  // Pad to full weeks
  const remainder = days.length % 7
  if (remainder !== 0) {
    const extra = 7 - remainder
    for (let i = 1; i <= extra; i++) {
      const d = new Date(year, month + 1, i)
      days.push({ date: padDate(d.getFullYear(), d.getMonth(), d.getDate()), day: d.getDate(), currentMonth: false })
    }
  }

  return days
})

/** Map dateStr → appointments */
const appointmentsByDate = computed(() => {
  const map: Record<string, Appointment[]> = {}
  for (const appt of appointments.value) {
    if (!map[appt.date]) map[appt.date] = []
    map[appt.date].push(appt)
  }
  return map
})

const selectedDayAppointments = computed(
  () => appointmentsByDate.value[selectedDate.value] ?? [],
)

function prevMonth() {
  if (viewMonth.value === 0) {
    viewMonth.value = 11
    viewYear.value--
  } else {
    viewMonth.value--
  }
}

function nextMonth() {
  if (viewMonth.value === 11) {
    viewMonth.value = 0
    viewYear.value++
  } else {
    viewMonth.value++
  }
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

/** Status → pill color */
function apptColor(status: string): string {
  const map: Record<string, string> = {
    scheduled: 'bg-ocean-100 text-ocean-700',
    completed: 'bg-sage-100 text-sage-700',
    cancelled: 'bg-warm-100 text-warm-500 line-through',
  }
  return map[status] ?? 'bg-warm-100 text-warm-600'
}

const isToday = (dateStr: string) => dateStr === today.toISOString().slice(0, 10)

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
  <div class="p-8 flex gap-6 min-h-0">
    <!-- Calendar area -->
    <div class="flex-1 min-w-0">
      <PageHeader title="Agenda">
        <button
          type="button"
          class="relative overflow-hidden bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-all hover:shadow-lg hover:shadow-sage-200"
          @click="openCreate(selectedDate)"
        >
          <Plus class="w-4 h-4" />
          Nuovo Appuntamento
        </button>
      </PageHeader>

      <!-- Month nav -->
      <div class="flex items-center justify-between mb-4">
        <button
          type="button"
          class="p-1.5 text-sage-500 hover:bg-sage-100 rounded-lg transition-colors"
          @click="prevMonth"
        >
          <ChevronLeft class="w-5 h-5" />
        </button>
        <h2 class="text-base font-semibold text-sage-900">
          {{ MONTH_NAMES[viewMonth] }} {{ viewYear }}
        </h2>
        <button
          type="button"
          class="p-1.5 text-sage-500 hover:bg-sage-100 rounded-lg transition-colors"
          @click="nextMonth"
        >
          <ChevronRight class="w-5 h-5" />
        </button>
      </div>

      <!-- Grid header -->
      <div class="grid grid-cols-7 mb-1">
        <div
          v-for="dayName in DAY_NAMES"
          :key="dayName"
          class="text-center text-xs font-medium text-sage-400 py-1"
        >
          {{ dayName }}
        </div>
      </div>

      <!-- Grid days -->
      <div class="grid grid-cols-7 gap-px rounded-xl overflow-hidden border border-sage-100" style="background: rgba(227,235,227,0.4)">
        <div
          v-for="cell in calendarDays"
          :key="cell.date"
          class="bg-white/80 min-h-[80px] p-1.5 cursor-pointer transition-colors"
          :class="[
            cell.date === selectedDate ? 'ring-2 ring-inset ring-sage-500 bg-sage-50/60' : 'hover:bg-sage-50/40',
            !cell.currentMonth ? 'opacity-40' : '',
          ]"
          @click="selectedDate = cell.date"
        >
          <!-- Day number -->
          <div class="flex items-center justify-between mb-1">
            <span
              class="text-xs font-medium w-6 h-6 flex items-center justify-center rounded-full"
              :class="isToday(cell.date)
                ? 'bg-gradient-to-br from-sage-600 to-ocean-500 text-white'
                : 'text-sage-700'"
            >
              {{ cell.day }}
            </span>
            <button
              type="button"
              class="w-5 h-5 text-sage-300 hover:text-sage-600 opacity-0 group-hover:opacity-100 transition-colors"
              @click.stop="openCreate(cell.date)"
            >
              <Plus class="w-3.5 h-3.5" />
            </button>
          </div>
          <!-- Appointment pills -->
          <div class="space-y-0.5">
            <div
              v-for="appt in (appointmentsByDate[cell.date] ?? []).slice(0, 3)"
              :key="appt.id"
              class="text-xs px-1.5 py-0.5 rounded truncate cursor-pointer"
              :class="apptColor(appt.status)"
              @click.stop="openEdit(appt)"
            >
              {{ appt.start_time.slice(0, 5) }} {{ appt.client_name }}
            </div>
            <div
              v-if="(appointmentsByDate[cell.date] ?? []).length > 3"
              class="text-xs text-sage-400 px-1"
            >
              + {{ (appointmentsByDate[cell.date] ?? []).length - 3 }} altri
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Side panel: selected day -->
    <div class="w-72 shrink-0">
      <div class="glass-card rounded-xl p-5 sticky top-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-sm font-semibold text-sage-900">
            {{ new Date(selectedDate + 'T00:00:00').toLocaleDateString('it-IT', {
              weekday: 'long',
              day: 'numeric',
              month: 'long',
            }) }}
          </h3>
          <button
            type="button"
            class="w-7 h-7 flex items-center justify-center text-sage-600 hover:bg-sage-50 rounded-lg transition-colors"
            title="Aggiungi appuntamento"
            @click="openCreate(selectedDate)"
          >
            <Plus class="w-4 h-4" />
          </button>
        </div>

        <div v-if="selectedDayAppointments.length === 0" class="text-sm text-sage-400 text-center py-6">
          Nessun appuntamento
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="appt in selectedDayAppointments"
            :key="appt.id"
            class="rounded-lg border border-sage-100 p-3 cursor-pointer hover:bg-sage-50/40 transition-colors"
            @click="openEdit(appt)"
          >
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs font-medium text-sage-500">
                {{ appt.start_time.slice(0, 5) }} – {{ appt.end_time.slice(0, 5) }}
              </span>
              <StatusBadge :status="appt.status" type="appointment" />
            </div>
            <p class="text-sm font-medium text-sage-900">{{ appt.client_name }}</p>
            <p v-if="appt.service_name" class="text-xs text-sage-500 mt-0.5">{{ appt.service_name }}</p>
            <p v-if="appt.notes" class="text-xs text-sage-400 mt-1 truncate">{{ appt.notes }}</p>
          </div>
        </div>
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
