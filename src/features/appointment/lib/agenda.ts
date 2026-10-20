/** The agenda's calendar grid and the figures of a day and a month. Months are counted from 0, as in `Date`. */
import type { Appointment, AppointmentStatus, UpdateAppointmentInput } from '@/features/appointment/model/appointment'
import { minutesBetween, parseIsoDate, toIsoDate } from '@/common/lib/format'

export interface CalendarCell {
  date: string
  day: number
  inMonth: boolean
  weekend: boolean
}

export interface MonthStats {
  total: number
  heldNotBilled: number
  toConfirm: number
  completed: number
}

/** Five or six weeks, Monday first, padded with the days of the neighbouring months. */
export function monthGrid(year: number, month: number): CalendarCell[] {
  const first = new Date(year, month, 1)
  const leading = (first.getDay() + 6) % 7
  const daysInMonth = new Date(year, month + 1, 0).getDate()
  const total = Math.ceil((leading + daysInMonth) / 7) * 7
  return Array.from({ length: total }, (_, index) => {
    const date = new Date(year, month, index - leading + 1)
    const weekday = (date.getDay() + 6) % 7
    return { date: toIsoDate(date), day: date.getDate(), inMonth: date.getMonth() === month, weekend: weekday >= 5 }
  })
}

/** The range loaded for a month: a week either side, so the padding days show their sessions. */
export function loadWindow(year: number, month: number): { from: string; to: string } {
  return { from: toIsoDate(new Date(year, month, -6)), to: toIsoDate(new Date(year, month + 1, 7)) }
}

/** Appointments grouped by date, each day in start-time order. */
export function appointmentsByDate(appointments: readonly Appointment[]): Map<string, Appointment[]> {
  const map = new Map<string, Appointment[]>()
  for (const appointment of appointments) {
    const list = map.get(appointment.date) ?? []
    list.push(appointment)
    map.set(appointment.date, list)
  }
  for (const list of map.values()) list.sort((a, b) => a.start_time.localeCompare(b.start_time))
  return map
}

/** Minutes of the day's sessions, cancelled ones aside. */
export function bookedMinutes(appointments: readonly Appointment[]): number {
  return appointments
    .filter((item) => item.status !== 'cancelled')
    .reduce((sum, item) => sum + minutesBetween(item.start_time, item.end_time), 0)
}

/** "Lunedì 5 ottobre". */
export function dayHeading(isoDate: string): string {
  const label = parseIsoDate(isoDate).toLocaleDateString('it-IT', { weekday: 'long', day: 'numeric', month: 'long' })
  return label.charAt(0).toUpperCase() + label.slice(1)
}

/** A month's live sessions: held but not yet invoiced, and past ones still waiting to be confirmed. */
export function monthStats(appointments: readonly Appointment[], year: number, month: number, today: string): MonthStats {
  const prefix = `${year}-${String(month + 1).padStart(2, '0')}`
  const live = appointments.filter((item) => item.status !== 'cancelled' && item.date.startsWith(prefix))
  const heldNotBilled = live.filter((item) => item.status === 'completed' && item.invoice_id === undefined).length
  const toConfirm = live.filter((item) => item.status === 'scheduled' && item.date < today).length
  const completed = live.filter((item) => item.status === 'completed').length
  return { total: live.length, heldNotBilled, toConfirm, completed }
}

/** The whole appointment sent back with a new status: the backend replaces every field. */
export function withAppointmentStatus(appointment: Appointment, status: AppointmentStatus): UpdateAppointmentInput {
  return {
    id: appointment.id,
    client_id: appointment.client_id,
    service_id: appointment.service_id,
    date: appointment.date,
    start_time: appointment.start_time,
    end_time: appointment.end_time,
    status,
    notes: appointment.notes,
    recurrence_group_id: appointment.recurrence_group_id,
  }
}

const DEFAULT_SESSION_MINUTES = 50

/**
 * The end time once the start field changed, given the new start and the end still in the form:
 * the length of that range, or 50 minutes when the start is not before the end.
 */
export function endTimeAfterStartChange(newStart: string, currentEnd: string): string {
  const length = minutesBetween(newStart, currentEnd) || DEFAULT_SESSION_MINUTES
  const [hours, minutes] = newStart.split(':').map(Number)
  const endMinutes = hours * 60 + minutes + length
  const pad = (value: number): string => String(value).padStart(2, '0')
  return `${pad(Math.floor(endMinutes / 60) % 24)}:${pad(endMinutes % 60)}`
}
