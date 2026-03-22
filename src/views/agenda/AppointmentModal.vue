<script setup lang="ts">
import { reactive, ref, watch, computed } from 'vue'
import { X, Trash2 } from 'lucide-vue-next'
import {
  createAppointment,
  createRecurringAppointments,
  updateAppointment,
  deleteAppointment,
} from '@/api'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import type {
  Appointment,
  AppointmentStatus,
  CreateRecurringAppointmentsInput,
} from '@/types'

const props = defineProps<{
  open: boolean
  date?: string
  appointment?: Appointment | null
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const clientsStore = useClientsStore()
const servicesStore = useServicesStore()

const saving = ref(false)
const formError = ref<string | null>(null)

const STATUS_OPTIONS: Array<{ value: AppointmentStatus; label: string }> = [
  { value: 'scheduled', label: 'Programmato' },
  { value: 'completed', label: 'Completato' },
  { value: 'cancelled', label: 'Annullato' },
]

const DAYS_OF_WEEK = [
  { value: 0, label: 'Dom' },
  { value: 1, label: 'Lun' },
  { value: 2, label: 'Mar' },
  { value: 3, label: 'Mer' },
  { value: 4, label: 'Gio' },
  { value: 5, label: 'Ven' },
  { value: 6, label: 'Sab' },
]

const form = reactive({
  client_id: 0,
  service_id: undefined as number | undefined,
  date: '',
  start_time: '09:00',
  end_time: '10:00',
  status: 'scheduled' as AppointmentStatus,
  notes: '',
})

const recurring = ref(false)
const recurringWeeks = ref(4)
const recurringDays = ref<number[]>([])

const isEdit = computed(() => !!props.appointment)

watch(
  () => props.open,
  (open) => {
    if (!open) return
    formError.value = null
    recurring.value = false
    recurringDays.value = []
    recurringWeeks.value = 4

    if (props.appointment) {
      Object.assign(form, {
        client_id: props.appointment.client_id,
        service_id: props.appointment.service_id,
        date: props.appointment.date,
        start_time: props.appointment.start_time,
        end_time: props.appointment.end_time,
        status: props.appointment.status,
        notes: props.appointment.notes,
      })
    } else {
      Object.assign(form, {
        client_id: 0,
        service_id: undefined,
        date: props.date ?? new Date().toISOString().slice(0, 10),
        start_time: '09:00',
        end_time: '10:00',
        status: 'scheduled',
        notes: '',
      })
    }
  },
  { immediate: true },
)

/**
 * Generates a list of ISO date strings for recurring days starting from base date.
 */
function generateRecurringDates(): string[] {
  if (recurringDays.value.length === 0) return []
  const base = new Date(form.date + 'T00:00:00')
  const dates: Set<string> = new Set()

  for (let week = 0; week < recurringWeeks.value; week++) {
    for (const dow of recurringDays.value) {
      const d = new Date(base)
      const diff = (dow - base.getDay() + 7) % 7 + week * 7
      d.setDate(base.getDate() + diff)
      if (d >= base) {
        dates.add(d.toISOString().slice(0, 10))
      }
    }
  }
  return Array.from(dates).sort()
}

async function onSubmit() {
  if (!form.client_id) {
    formError.value = 'Seleziona un cliente.'
    return
  }

  saving.value = true
  formError.value = null

  try {
    if (isEdit.value && props.appointment) {
      await updateAppointment({
        id: props.appointment.id,
        client_id: form.client_id,
        service_id: form.service_id,
        date: form.date,
        start_time: form.start_time,
        end_time: form.end_time,
        status: form.status,
        notes: form.notes,
      })
    } else if (recurring.value && recurringDays.value.length > 0) {
      const recurringInput: CreateRecurringAppointmentsInput = {
        client_id: form.client_id,
        service_id: form.service_id,
        dates: generateRecurringDates(),
        start_time: form.start_time,
        end_time: form.end_time,
        notes: form.notes,
      }
      await createRecurringAppointments(recurringInput)
    } else {
      await createAppointment({
        client_id: form.client_id,
        service_id: form.service_id,
        date: form.date,
        start_time: form.start_time,
        end_time: form.end_time,
        status: form.status,
        notes: form.notes,
      })
    }
    emit('saved')
    emit('close')
  } catch (e) {
    formError.value = String(e)
  } finally {
    saving.value = false
  }
}

async function onDelete() {
  if (!props.appointment) return
  saving.value = true
  try {
    await deleteAppointment(props.appointment.id)
    emit('saved')
    emit('close')
  } catch (e) {
    formError.value = String(e)
  } finally {
    saving.value = false
  }
}

function toggleRecurringDay(day: number) {
  const idx = recurringDays.value.indexOf(day)
  if (idx === -1) {
    recurringDays.value.push(day)
  } else {
    recurringDays.value.splice(idx, 1)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" @click="emit('close')" />
      <div class="relative glass-card rounded-2xl shadow-2xl w-full max-w-lg mx-4 p-6 max-h-[90vh] overflow-y-auto animate-in">
        <div class="flex items-center justify-between mb-5">
          <h3 class="text-base font-semibold text-sage-900">
            {{ isEdit ? 'Modifica Appuntamento' : 'Nuovo Appuntamento' }}
          </h3>
          <button type="button" class="text-sage-400 hover:text-sage-600 transition-colors" @click="emit('close')">
            <X class="w-5 h-5" />
          </button>
        </div>

        <form class="space-y-4" @submit.prevent="onSubmit">
          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Cliente *</label>
            <select
              v-model.number="form.client_id"
              required
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
            >
              <option :value="0" disabled>— Seleziona cliente —</option>
              <option
                v-for="client in clientsStore.clients"
                :key="client.id"
                :value="client.id"
              >
                {{ client.last_name }} {{ client.first_name }}
              </option>
            </select>
          </div>

          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Prestazione</label>
            <select
              v-model.number="form.service_id"
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
            >
              <option :value="undefined">— nessuna —</option>
              <option
                v-for="service in servicesStore.services"
                :key="service.id"
                :value="service.id"
              >
                {{ service.name }}
              </option>
            </select>
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div>
              <label class="text-sm font-medium text-sage-700 block mb-1">Data *</label>
              <input
                v-model="form.date"
                type="date"
                required
                class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-sage-700 block mb-1">Inizio</label>
              <input
                v-model="form.start_time"
                type="time"
                required
                class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-sage-700 block mb-1">Fine</label>
              <input
                v-model="form.end_time"
                type="time"
                required
                class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
              />
            </div>
          </div>

          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Stato</label>
            <select
              v-model="form.status"
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
            >
              <option v-for="opt in STATUS_OPTIONS" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>

          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Note</label>
            <textarea
              v-model="form.notes"
              rows="2"
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80 resize-none"
            />
          </div>

          <!-- Recurring section (only on create) -->
          <template v-if="!isEdit">
            <div class="border-t border-sage-100 pt-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="recurring"
                  type="checkbox"
                  class="rounded border-sage-300 text-sage-600 focus:ring-sage-400"
                />
                <span class="text-sm font-medium text-sage-700">Appuntamento ricorrente</span>
              </label>

              <div v-if="recurring" class="mt-3 space-y-3">
                <div>
                  <label class="text-xs font-medium text-sage-500 block mb-2">Giorni della settimana</label>
                  <div class="flex gap-1.5">
                    <button
                      v-for="day in DAYS_OF_WEEK"
                      :key="day.value"
                      type="button"
                      class="px-2.5 py-1 rounded-lg text-xs font-medium border transition-colors"
                      :class="recurringDays.includes(day.value)
                        ? 'bg-gradient-to-r from-sage-600 to-ocean-500 text-white border-transparent'
                        : 'border-sage-200 text-sage-600 hover:bg-sage-50'"
                      @click="toggleRecurringDay(day.value)"
                    >
                      {{ day.label }}
                    </button>
                  </div>
                </div>
                <div>
                  <label class="text-xs font-medium text-sage-500 block mb-1">Numero di settimane</label>
                  <input
                    v-model.number="recurringWeeks"
                    type="number"
                    min="1"
                    max="52"
                    class="w-24 border border-sage-200 rounded-lg px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                  />
                </div>
              </div>
            </div>
          </template>

          <div v-if="formError" class="rounded-lg bg-red-50 border border-red-200 px-3 py-2 text-sm text-red-700">
            {{ formError }}
          </div>

          <div class="flex items-center gap-3 pt-2">
            <button
              v-if="isEdit"
              type="button"
              :disabled="saving"
              class="bg-red-600 text-white hover:bg-red-700 px-3 py-2 rounded-lg text-sm font-medium flex items-center gap-1.5 transition-colors disabled:opacity-60"
              @click="onDelete"
            >
              <Trash2 class="w-4 h-4" />
              Elimina
            </button>
            <div class="flex-1" />
            <button
              type="button"
              class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              @click="emit('close')"
            >
              Annulla
            </button>
            <button
              type="submit"
              :disabled="saving"
              class="group relative overflow-hidden text-white font-semibold px-4 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 disabled:opacity-60 cursor-pointer focus:outline-none"
              style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
            >
              <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
              <span class="relative z-10">{{ saving ? 'Salvataggio...' : 'Salva' }}</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>
</template>
