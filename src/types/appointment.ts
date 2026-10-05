export type AppointmentStatus = 'scheduled' | 'completed' | 'cancelled'

export interface CreateAppointmentInput {
  client_id: number
  service_id?: number
  date: string
  start_time: string
  end_time: string
  status: AppointmentStatus
  notes: string
  recurrence_group_id?: number
}

export interface UpdateAppointmentInput extends CreateAppointmentInput {
  id: number
}

export interface CreateRecurringAppointmentsInput {
  client_id: number
  service_id?: number
  dates: string[]
  start_time: string
  end_time: string
  notes: string
}

export interface Appointment {
  id: number
  client_id: number
  client_name: string
  service_id?: number
  service_name?: string
  date: string
  start_time: string
  end_time: string
  status: AppointmentStatus
  notes: string
  recurrence_group_id?: number
  invoice_id?: number
  created_at: string
  updated_at: string
}
