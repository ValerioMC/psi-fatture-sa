/**
 * Changes an appointment's status in place. Marking a session held never asks
 * first: the toast offers "Annulla" instead, which restores the previous status.
 */
import type { Ref } from 'vue'
import { updateAppointment } from '@/api'
import { useToastStore } from '@/stores/toast'
import type { Appointment, AppointmentStatus } from '@/types'
import { withAppointmentStatus } from '@/utils/agenda'

export function useAppointmentStatus(appointments: Ref<Appointment[]>) {
  const toast = useToastStore()

  async function setStatus(appointment: Appointment, status: AppointmentStatus, announce = true): Promise<void> {
    const previous = appointment.status
    try {
      const updated = await updateAppointment(withAppointmentStatus(appointment, status))
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

  return { setStatus }
}
