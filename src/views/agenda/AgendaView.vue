<script setup lang="ts">
/**
 * The month on the left, the chosen day on the right. A session that has
 * happened is marked "svolta" with one click from the day list: that is what
 * makes it appear in the monthly invoicing, so it must never take a dialog.
 */
import { computed, onMounted, ref, watch } from 'vue'
import { CalendarDays, Check, Clock, Plus, Repeat } from 'lucide-vue-next'
import { listAppointments, updateAppointment } from '@/api'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import { useToastStore } from '@/stores/toast'
import type { Appointment, AppointmentStatus } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import PeriodStepper from '@/components/ui/PeriodStepper.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import AppointmentModal from './AppointmentModal.vue'
import { formatMonthYear, minutesBetween, parseIsoDate, toIsoDate, todayIso } from '@/utils/format'
import { plural } from '@/utils/labels'

const clientsStore = useClientsStore()
const servicesStore = useServicesStore()
const toast = useToastStore()

const today = todayIso()
const viewYear = ref(new Date().getFullYear())
const viewMonth = ref(new Date().getMonth())
const selectedDate = ref(today)
const appointments = ref<Appointment[]>([])
const loading = ref(false)

const modalOpen = ref(false)
const modalDate = ref<string | undefined>(undefined)
const editing = ref<Appointment | null>(null)

const WEEKDAYS = ['lun', 'mar', 'mer', 'gio', 'ven', 'sab', 'dom']

interface CalendarCell {
  date: string
  day: number
  inMonth: boolean
  weekend: boolean
}

/** Six-by-seven or five-by-seven grid, Monday first, padded with the neighbouring months. */
const cells = computed<CalendarCell[]>(() => {
  const first = new Date(viewYear.value, viewMonth.value, 1)
  const leading = (first.getDay() + 6) % 7
  const daysInMonth = new Date(viewYear.value, viewMonth.value + 1, 0).getDate()
  const total = Math.ceil((leading + daysInMonth) / 7) * 7
  return Array.from({ length: total }, (_, index) => {
    const date = new Date(viewYear.value, viewMonth.value, index - leading + 1)
    const weekday = (date.getDay() + 6) % 7
    return { date: toIsoDate(date), day: date.getDate(), inMonth: date.getMonth() === viewMonth.value, weekend: weekday >= 5 }
  })
})

const byDate = computed(() => {
  const map = new Map<string, Appointment[]>()
  for (const appointment of appointments.value) {
    const list = map.get(appointment.date) ?? []
    list.push(appointment)
    map.set(appointment.date, list)
  }
  for (const list of map.values()) list.sort((a, b) => a.start_time.localeCompare(b.start_time))
  return map
})

const dayList = computed(() => byDate.value.get(selectedDate.value) ?? [])
const dayLabel = computed(() => {
  const label = parseIsoDate(selectedDate.value).toLocaleDateString('it-IT', { weekday: 'long', day: 'numeric', month: 'long' })
  return label.charAt(0).toUpperCase() + label.slice(1)
})
const dayMinutes = computed(() =>
  dayList.value.filter((item) => item.status !== 'cancelled').reduce((sum, item) => sum + minutesBetween(item.start_time, item.end_time), 0),
)

const monthStats = computed(() => {
  const prefix = `${viewYear.value}-${String(viewMonth.value + 1).padStart(2, '0')}`
  const live = appointments.value.filter((item) => item.status !== 'cancelled' && item.date.startsWith(prefix))
  const heldNotBilled = live.filter((item) => item.status === 'completed' && item.invoice_id === undefined).length
  const toConfirm = live.filter((item) => item.status === 'scheduled' && item.date < today).length
  const completed = live.filter((item) => item.status === 'completed').length
  return { total: live.length, heldNotBilled, toConfirm, completed }
})

async function load(): Promise<void> {
  loading.value = true
  const from = toIsoDate(new Date(viewYear.value, viewMonth.value, -6))
  const to = toIsoDate(new Date(viewYear.value, viewMonth.value + 1, 7))
  try {
    appointments.value = await listAppointments(from, to)
  } catch (error) {
    toast.notifyError(error, 'Agenda non disponibile')
  } finally {
    loading.value = false
  }
}

function shiftMonth(delta: number): void {
  const next = new Date(viewYear.value, viewMonth.value + delta, 1)
  viewYear.value = next.getFullYear()
  viewMonth.value = next.getMonth()
}

function goToday(): void {
  const now = new Date()
  viewYear.value = now.getFullYear()
  viewMonth.value = now.getMonth()
  selectedDate.value = today
}

function selectCell(cell: CalendarCell): void {
  selectedDate.value = cell.date
  if (!cell.inMonth) shiftMonth(cell.date < toIsoDate(new Date(viewYear.value, viewMonth.value, 1)) ? -1 : 1)
}

function openCreate(): void {
  editing.value = null
  modalDate.value = selectedDate.value
  modalOpen.value = true
}

function openEdit(appointment: Appointment): void {
  editing.value = appointment
  modalDate.value = appointment.date
  modalOpen.value = true
}

async function setStatus(appointment: Appointment, status: AppointmentStatus, announce = true): Promise<void> {
  const previous = appointment.status
  try {
    const updated = await updateAppointment({
      id: appointment.id,
      client_id: appointment.client_id,
      service_id: appointment.service_id,
      date: appointment.date,
      start_time: appointment.start_time,
      end_time: appointment.end_time,
      status,
      notes: appointment.notes,
      recurrence_group_id: appointment.recurrence_group_id,
    })
    appointments.value = appointments.value.map((item) => (item.id === updated.id ? { ...item, ...updated } : item))
    if (announce) {
      toast.notify(`Seduta con ${appointment.client_name} segnata come svolta`, {
        label: 'Annulla',
        run: () => setStatus({ ...appointment, status }, previous, false),
      })
    }
  } catch (error) {
    toast.notifyError(error, 'Aggiornamento non riuscito')
  }
}

/** A session in a day cell: a soft slip of its status colour, so a busy week reads as texture at a glance. */
const CHIP_SLIP: Record<AppointmentStatus, string> = {
  scheduled: 'bg-accent-soft text-text-muted',
  completed: 'bg-safe-soft text-text-muted',
  cancelled: 'text-text-subtle line-through',
}

const CHIP_DOT: Record<AppointmentStatus, string> = {
  scheduled: 'bg-accent',
  completed: 'bg-safe',
  cancelled: 'bg-text-subtle',
}

watch([viewYear, viewMonth], load)
onMounted(() => Promise.all([clientsStore.fetchClients(), servicesStore.fetchServices(false), load()]))
</script>

<template>
  <div>
    <PageHeader title="Agenda" :icon="CalendarDays">
      <AppButton variant="primary" :icon="Plus" @click="openCreate">Nuovo appuntamento</AppButton>
    </PageHeader>

    <div class="page pt-6 pb-12">
      <div class="grid grid-cols-1 items-start gap-5 xl:grid-cols-[minmax(0,1fr)_22rem] 2xl:grid-cols-[minmax(0,1fr)_25rem]">
        <!-- ── Month ─────────────────────────────────────────────────────── -->
        <AppCard :padded="false" class="settle overflow-hidden">
          <div class="flex items-center justify-between gap-4 px-5 py-4">
            <div>
              <h2 class="display text-2xl text-text">{{ formatMonthYear(viewYear, viewMonth + 1) }}</h2>
              <p class="mt-1 flex flex-wrap items-center gap-1.5 text-sm text-text-subtle">
                <AppBadge tone="accent">{{ plural(monthStats.total, 'seduta', 'sedute') }}</AppBadge>
                <AppBadge v-if="monthStats.completed > 0" tone="safe" dot>{{ monthStats.completed }} svolte</AppBadge>
                <AppBadge v-if="monthStats.toConfirm > 0" tone="warn" dot>{{ monthStats.toConfirm }} da confermare</AppBadge>
              </p>
            </div>
            <div class="flex items-center gap-2">
              <AppButton size="sm" @click="goToday">Oggi</AppButton>
              <PeriodStepper
                :label="''"
                min-width="0"
                previous-label="Mese precedente"
                next-label="Mese successivo"
                @previous="shiftMonth(-1)"
                @next="shiftMonth(1)"
              />
            </div>
          </div>

          <div class="grid grid-cols-7 border-y border-border bg-[color-mix(in_srgb,var(--surface-sunken)_55%,var(--surface-raised))] text-xs font-medium text-text-subtle" aria-hidden="true">
            <span v-for="(weekday, index) in WEEKDAYS" :key="weekday" class="px-2.5 py-2" :class="index >= 5 ? 'text-text-subtle/70' : ''">{{ weekday }}</span>
          </div>

          <div class="grid grid-cols-7" :class="loading ? 'opacity-60 transition-opacity' : ''" role="grid" :aria-label="formatMonthYear(viewYear, viewMonth + 1)">
            <button
              v-for="(cell, index) in cells"
              :key="cell.date"
              type="button"
              class="group relative flex min-h-[max(6.25rem,calc((100vh-18.5rem)/6))] flex-col items-stretch gap-1 border-border p-1.5 text-left transition-colors focus-ring focus-visible:z-[1]"
              :class="[
                index % 7 !== 6 ? 'border-r' : '',
                index < cells.length - 7 ? 'border-b' : '',
                cell.date === selectedDate ? 'bg-accent-soft' : cell.weekend || !cell.inMonth ? 'bg-surface hover:bg-surface-hover' : 'hover:bg-surface-hover',
              ]"
              :aria-pressed="cell.date === selectedDate"
              :aria-label="`${parseIsoDate(cell.date).toLocaleDateString('it-IT', { weekday: 'long', day: 'numeric', month: 'long' })}, ${plural((byDate.get(cell.date) ?? []).length, 'appuntamento', 'appuntamenti')}`"
              @click="selectCell(cell)"
            >
              <span
                v-if="cell.date === selectedDate"
                class="pointer-events-none absolute inset-0 ring-1 ring-inset ring-accent-line"
                aria-hidden="true"
              />
              <span
                class="tabular grid size-6 place-items-center rounded-full text-sm"
                :class="cell.date === today
                  ? 'bg-accent font-semibold text-accent-ink'
                  : cell.inMonth ? 'text-text' : 'text-text-subtle/70'"
              >{{ cell.day }}</span>
              <span
                v-for="appointment in (byDate.get(cell.date) ?? []).slice(0, 3)"
                :key="appointment.id"
                class="flex items-center gap-1.5 truncate rounded-[5px] px-1.5 text-2xs leading-[1.25rem]"
                :class="[CHIP_SLIP[appointment.status], cell.inMonth ? '' : 'opacity-60']"
              >
                <span class="size-1.5 shrink-0 rounded-full" :class="CHIP_DOT[appointment.status]" aria-hidden="true" />
                <span class="tabular hidden shrink-0 text-text-subtle 2xl:inline">{{ appointment.start_time.slice(0, 5) }}</span>
                <span class="truncate" :title="`${appointment.start_time.slice(0, 5)} ${appointment.client_name}`">{{ appointment.client_name.split(' ')[0] }}</span>
              </span>
              <span v-if="(byDate.get(cell.date) ?? []).length > 3" class="px-1 text-2xs text-text-subtle">
                +{{ (byDate.get(cell.date) ?? []).length - 3 }} {{ (byDate.get(cell.date) ?? []).length - 3 === 1 ? 'altro' : 'altri' }}
              </span>
            </button>
          </div>
        </AppCard>

        <!-- ── Day ───────────────────────────────────────────────────────── -->
        <aside class="settle xl:sticky xl:top-24" style="--settle: 1">
          <AppCard :padded="false">
            <div class="flex items-start justify-between gap-3 border-b border-border px-5 py-4">
              <Transition name="swap" mode="out-in">
                <div :key="selectedDate">
                  <h2 class="text-lg font-medium text-text">{{ dayLabel }}</h2>
                  <p class="text-sm text-text-subtle">
                    <template v-if="dayList.length > 0">{{ plural(dayList.length, 'appuntamento', 'appuntamenti') }} · {{ Math.floor(dayMinutes / 60) }} h {{ dayMinutes % 60 > 0 ? `${dayMinutes % 60} min` : '' }}</template>
                    <template v-else>Giornata libera</template>
                  </p>
                </div>
              </Transition>
              <AppButton size="sm" variant="ghost" :icon="Plus" label="Aggiungi appuntamento in questa data" @click="openCreate" />
            </div>

            <Transition name="swap" mode="out-in">
              <ol v-if="dayList.length > 0" :key="selectedDate" class="max-h-[calc(100vh-18rem)] overflow-y-auto p-2" data-lenis-prevent>
                <li v-for="appointment in dayList" :key="appointment.id">
                  <div
                    class="group flex cursor-pointer gap-3 rounded-control px-3 py-3 transition-colors hover:bg-surface-hover"
                    :class="appointment.status === 'cancelled' ? 'opacity-60' : ''"
                    role="button"
                    tabindex="0"
                    @click="openEdit(appointment)"
                    @keydown.enter="openEdit(appointment)"
                  >
                    <div class="tabular w-11 shrink-0 pt-0.5 text-right">
                      <p class="text-base font-medium text-text">{{ appointment.start_time.slice(0, 5) }}</p>
                      <p class="text-2xs text-text-subtle">{{ appointment.end_time.slice(0, 5) }}</p>
                    </div>
                    <span class="w-0.5 shrink-0 self-stretch rounded-full" :class="CHIP_DOT[appointment.status]" aria-hidden="true" />
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <PatientMonogram :name="appointment.client_name" size="sm" />
                        <p class="truncate text-base font-medium text-text" :class="appointment.status === 'cancelled' ? 'line-through' : ''">{{ appointment.client_name }}</p>
                      </div>
                      <p v-if="appointment.service_name" class="mt-1 truncate text-sm text-text-muted">{{ appointment.service_name }}</p>
                      <div class="mt-1.5 flex items-center gap-2 text-xs text-text-subtle">
                        <span class="inline-flex items-center gap-1"><Clock :size="12" aria-hidden="true" />{{ minutesBetween(appointment.start_time, appointment.end_time) }} min</span>
                        <span v-if="appointment.recurrence_group_id" class="inline-flex items-center gap-1"><Repeat :size="12" aria-hidden="true" />ricorrente</span>
                        <span v-if="appointment.invoice_id" class="text-safe">fatturata</span>
                      </div>
                      <p v-if="appointment.notes" class="mt-1 truncate text-xs italic text-text-subtle">{{ appointment.notes }}</p>
                    </div>
                    <div class="flex shrink-0 flex-col items-end gap-2" @click.stop>
                      <StatusBadge v-if="appointment.status !== 'scheduled'" type="appointment" :status="appointment.status" />
                      <AppButton
                        v-if="appointment.status === 'scheduled' && appointment.date <= today"
                        size="sm"
                        :icon="Check"
                        @click="setStatus(appointment, 'completed')"
                      >
                        Svolta
                      </AppButton>
                    </div>
                  </div>
                </li>
              </ol>
              <div v-else :key="`${selectedDate}-empty`" class="flex flex-col items-center px-6 py-12 text-center">
                <CalendarDays :size="24" :stroke-width="1.5" class="text-text-subtle" aria-hidden="true" />
                <p class="mt-2 text-sm text-text-muted">Nessun appuntamento in questa data.</p>
                <AppButton class="mt-4" size="sm" :icon="Plus" @click="openCreate">Aggiungi</AppButton>
              </div>
            </Transition>
          </AppCard>

          <p v-if="monthStats.heldNotBilled > 0" class="mt-3 px-1 text-xs text-text-subtle">
            {{ plural(monthStats.heldNotBilled, 'seduta svolta', 'sedute svolte') }} ancora da fatturare questo mese.
            <RouterLink :to="{ path: '/invoices/monthly', query: { year: viewYear, month: viewMonth + 1 } }" class="font-medium text-accent hover:underline underline-offset-2">Fatturazione mensile</RouterLink>
          </p>
        </aside>
      </div>
    </div>

    <AppointmentModal
      :open="modalOpen"
      :date="modalDate"
      :appointment="editing"
      @close="modalOpen = false"
      @saved="load"
    />
  </div>
</template>
