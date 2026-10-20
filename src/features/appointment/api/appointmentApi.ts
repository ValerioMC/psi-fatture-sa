/** Agenda appointments. */
import { invoke } from '@tauri-apps/api/core'
import type { Appointment, CreateAppointmentInput, CreateRecurringAppointmentsInput, UpdateAppointmentInput } from '@/features/appointment/model/appointment'

export const listAppointments = (
  dateFrom?: string,
  dateTo?: string,
  clientId?: number,
) =>
  invoke<Appointment[]>('list_appointments', {
    dateFrom: dateFrom ?? null,
    dateTo: dateTo ?? null,
    clientId: clientId ?? null,
  })
export const createAppointment = (input: CreateAppointmentInput) =>
  invoke<Appointment>('create_appointment', { input })
export const createRecurringAppointments = (input: CreateRecurringAppointmentsInput) =>
  invoke<Appointment[]>('create_recurring_appointments', { input })
export const updateAppointment = (input: UpdateAppointmentInput) =>
  invoke<Appointment>('update_appointment', { input })
export const deleteAppointment = (id: number) => invoke<void>('delete_appointment', { id })
