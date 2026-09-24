<script setup lang="ts">
/**
 * Create or edit one appointment; on create it can repeat weekly on chosen
 * weekdays. Moving the start time moves the end with it, keeping the length.
 */
import { computed, reactive, ref, watch } from 'vue'
import { Trash2 } from 'lucide-vue-next'
import { createAppointment, createRecurringAppointments, deleteAppointment, updateAppointment } from '@/api'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import { useToastStore } from '@/stores/toast'
import type { Appointment, AppointmentStatus } from '@/types'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppButton from '@/components/ui/AppButton.vue'
import FormField from '@/components/ui/FormField.vue'
import ComboBox from '@/components/ui/ComboBox.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import type { ComboOption, SegmentOption } from '@/components/ui/types'
import { clientDisplayName } from '@/utils/client'
import { formatDateLong, minutesBetween, todayIso } from '@/utils/format'
import { APPOINTMENT_STATUS, APPOINTMENT_STATUS_ORDER, plural } from '@/utils/labels'
import { weeklyDates } from '@/utils/recurrence'

const props = defineProps<{
  open: boolean
  date?: string
  appointment?: Appointment | null
}>()

const emit = defineEmits<{ close: []; saved: [] }>()

const clientsStore = useClientsStore()
const servicesStore = useServicesStore()
const toast = useToastStore()

const saving = ref(false)
const deleteOpen = ref(false)
const deleting = ref(false)
const errors = reactive<{ client?: string; time?: string; days?: string }>({})

const form = reactive({
  client_id: null as number | null,
  service_id: undefined as number | undefined,
  date: '',
  start_time: '09:00',
  end_time: '09:50',
  status: 'scheduled' as AppointmentStatus,
  notes: '',
})

const recurring = ref(false)
const weeks = ref(8)
const weekdays = ref<number[]>([])

const isEdit = computed(() => props.appointment != null)

/** Monday first, as on the calendar; values are Date#getDay numbers. */
const WEEKDAY_CHIPS = [
  { value: 1, label: 'Lun' }, { value: 2, label: 'Mar' }, { value: 3, label: 'Mer' },
  { value: 4, label: 'Gio' }, { value: 5, label: 'Ven' }, { value: 6, label: 'Sab' }, { value: 0, label: 'Dom' },
]

const STATUS_OPTIONS: SegmentOption<AppointmentStatus>[] = APPOINTMENT_STATUS_ORDER.map((value) => ({
  value,
  label: APPOINTMENT_STATUS[value].label,
  tone: APPOINTMENT_STATUS[value].tone,
}))

const clientOptions = computed<ComboOption[]>(() =>
  clientsStore.clients.map((client) => ({ value: client.id, label: clientDisplayName(client), detail: client.fiscal_code })),
)

const recurringDates = computed(() => (recurring.value ? weeklyDates(form.date, weekdays.value, weeks.value) : []))

watch(
  () => props.open,
  (open) => {
    if (!open) return
    for (const key of Object.keys(errors) as (keyof typeof errors)[]) delete errors[key]
    recurring.value = false
    weeks.value = 8
    const source = props.appointment
    if (source) {
      Object.assign(form, {
        client_id: source.client_id,
        service_id: source.service_id,
        date: source.date,
        start_time: source.start_time.slice(0, 5),
        end_time: source.end_time.slice(0, 5),
        status: source.status,
        notes: source.notes,
      })
    } else {
      Object.assign(form, { client_id: null, service_id: undefined, date: props.date ?? todayIso(), start_time: '09:00', end_time: '09:50', status: 'scheduled', notes: '' })
    }
    const weekday = new Date(`${form.date}T12:00:00`).getDay()
    weekdays.value = [weekday]
  },
  { immediate: true },
)

/** Keeps the session length when the start moves (50 minutes when there is none yet). */
function onStartChange(): void {
  const length = minutesBetween(form.start_time, form.end_time) || 50
  const [hours, minutes] = form.start_time.split(':').map(Number)
  const end = hours * 60 + minutes + length
  form.end_time = `${String(Math.floor(end / 60) % 24).padStart(2, '0')}:${String(end % 60).padStart(2, '0')}`
}

function toggleWeekday(value: number): void {
  weekdays.value = weekdays.value.includes(value)
    ? weekdays.value.filter((day) => day !== value)
    : [...weekdays.value, value]
}

function validate(): boolean {
  for (const key of Object.keys(errors) as (keyof typeof errors)[]) delete errors[key]
  if (form.client_id === null) errors.client = 'Scegli il paziente.'
  if (minutesBetween(form.start_time, form.end_time) <= 0) errors.time = 'La fine deve venire dopo l’inizio.'
  if (recurring.value && weekdays.value.length === 0) errors.days = 'Scegli almeno un giorno.'
  return Object.keys(errors).length === 0
}

async function onSubmit(): Promise<void> {
  if (!validate() || form.client_id === null) return
  saving.value = true
  const base = {
    client_id: form.client_id,
    service_id: form.service_id,
    start_time: form.start_time,
    end_time: form.end_time,
    notes: form.notes,
  }
  try {
    if (props.appointment) {
      await updateAppointment({ ...base, id: props.appointment.id, date: form.date, status: form.status, recurrence_group_id: props.appointment.recurrence_group_id })
      toast.notify('Appuntamento aggiornato')
    } else if (recurring.value) {
      const created = await createRecurringAppointments({ ...base, dates: recurringDates.value })
      toast.notify(`${plural(created.length, 'appuntamento creato', 'appuntamenti creati')}`)
    } else {
      await createAppointment({ ...base, date: form.date, status: form.status })
      toast.notify('Appuntamento creato')
    }
    emit('saved')
    emit('close')
  } catch (error) {
    toast.notifyError(error, 'Salvataggio non riuscito')
  } finally {
    saving.value = false
  }
}

async function confirmDelete(): Promise<void> {
  if (!props.appointment) return
  deleting.value = true
  try {
    await deleteAppointment(props.appointment.id)
    toast.notify('Appuntamento eliminato')
    deleteOpen.value = false
    emit('saved')
    emit('close')
  } catch (error) {
    toast.notifyError(error, 'Eliminazione non riuscita')
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <AppDialog :open="open && !deleteOpen" :title="isEdit ? 'Modifica appuntamento' : 'Nuovo appuntamento'" @close="emit('close')">
    <form id="appointment-form" class="space-y-5" novalidate @submit.prevent="onSubmit">
      <FormField v-slot="{ id, invalid, describedBy }" label="Paziente" required :error="errors.client">
        <ComboBox :id="id" v-model="form.client_id" :options="clientOptions" placeholder="Cerca per cognome" :invalid="invalid" :described-by="describedBy" />
      </FormField>

      <FormField v-slot="{ id }" label="Prestazione" optional>
        <select :id="id" v-model="form.service_id" class="field">
          <option :value="undefined">Nessuna</option>
          <option v-for="service in servicesStore.services.filter((item) => item.is_active || item.id === form.service_id)" :key="service.id" :value="service.id">{{ service.name }}</option>
        </select>
      </FormField>

      <div class="grid grid-cols-[1fr_7.5rem_7.5rem] gap-3">
        <FormField v-slot="{ id }" label="Data" required>
          <input :id="id" v-model="form.date" type="date" class="field" />
        </FormField>
        <FormField v-slot="{ id }" label="Inizio">
          <input :id="id" v-model="form.start_time" type="time" step="300" class="field tabular" @change="onStartChange" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="Fine" :error="errors.time">
          <input :id="id" v-model="form.end_time" type="time" step="300" class="field tabular" :aria-invalid="invalid" :aria-describedby="describedBy" />
        </FormField>
      </div>

      <div v-if="isEdit || !recurring">
        <p class="mb-1.5 text-sm font-medium text-text-muted">Stato</p>
        <SegmentedControl v-model="form.status" :options="STATUS_OPTIONS" label="Stato dell'appuntamento" block />
      </div>

      <FormField v-slot="{ id }" label="Note" optional>
        <textarea :id="id" v-model="form.notes" rows="2" class="field" />
      </FormField>

      <div v-if="!isEdit" class="rounded-control border border-border bg-surface p-4">
        <ToggleSwitch v-model="recurring" label="Si ripete ogni settimana" description="Crea in un colpo tutte le sedute del percorso." />
        <div v-if="recurring" class="mt-4 space-y-4 pl-12">
          <div>
            <div class="flex flex-wrap gap-1.5" role="group" aria-label="Giorni della settimana">
              <button
                v-for="chip in WEEKDAY_CHIPS"
                :key="chip.value"
                type="button"
                class="h-8 w-11 rounded-control border text-sm font-medium transition-colors focus-ring"
                :class="weekdays.includes(chip.value) ? 'border-accent bg-accent text-accent-ink' : 'border-border-strong bg-surface-raised text-text-muted hover:text-text'"
                :aria-pressed="weekdays.includes(chip.value)"
                @click="toggleWeekday(chip.value)"
              >
                {{ chip.label }}
              </button>
            </div>
            <p v-if="errors.days" class="mt-1.5 text-xs text-danger" role="alert">{{ errors.days }}</p>
          </div>
          <label class="flex items-center gap-2 text-sm text-text-muted">
            Per
            <input v-model.number="weeks" type="number" min="1" max="52" class="field field-sm tabular w-16 text-center" aria-label="Numero di settimane" />
            settimane
          </label>
          <p v-if="recurringDates.length > 0" class="text-sm text-text-muted">
            {{ plural(recurringDates.length, 'seduta', 'sedute') }}, dal {{ formatDateLong(recurringDates[0]) }} al {{ formatDateLong(recurringDates[recurringDates.length - 1]) }}.
          </p>
        </div>
      </div>
    </form>

    <template #footer>
      <AppButton v-if="isEdit" variant="danger-quiet" :icon="Trash2" @click="deleteOpen = true">Elimina</AppButton>
      <div class="flex-1" />
      <AppButton variant="ghost" @click="emit('close')">Annulla</AppButton>
      <AppButton variant="primary" type="submit" form="appointment-form" :loading="saving">
        {{ isEdit ? 'Salva' : recurring ? `Crea ${plural(recurringDates.length, 'seduta', 'sedute')}` : 'Crea' }}
      </AppButton>
    </template>
  </AppDialog>

  <ConfirmDialog
    :open="deleteOpen"
    title="Eliminare l'appuntamento?"
    :blast-radius="appointment ? `La seduta con ${appointment.client_name} del ${formatDateLong(appointment.date)} alle ${appointment.start_time.slice(0, 5)} verrà eliminata.` : undefined"
    :loading="deleting"
    @confirm="confirmDelete"
    @cancel="deleteOpen = false"
  />
</template>
