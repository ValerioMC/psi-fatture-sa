import { describe, expect, it } from 'vitest'
import type { Appointment, AppointmentStatus } from '@/types'
import { appointmentsByDate, bookedMinutes, dayHeading, loadWindow, monthGrid, monthStats, withAppointmentStatus } from './agenda'

const appointment = (id: number, date: string, start: string, end: string, status: AppointmentStatus, invoice_id?: number) =>
  ({ id, client_id: 1, client_name: 'Anna', date, start_time: start, end_time: end, status, notes: 'n', invoice_id }) as Appointment

describe('monthGrid', () => {
  it('starts on Monday and pads with the neighbouring months', () => {
    const cells = monthGrid(2026, 9)
    expect(cells).toHaveLength(35)
    expect(cells[0]).toEqual({ date: '2026-09-28', day: 28, inMonth: false, weekend: false })
    expect(cells[3]).toEqual({ date: '2026-10-01', day: 1, inMonth: true, weekend: false })
    expect(cells[5].weekend).toBe(true)
    expect(cells.at(-1)?.date).toBe('2026-11-01')
  })
})

describe('loadWindow', () => {
  it('reaches a week into the months either side', () => {
    expect(loadWindow(2026, 9)).toEqual({ from: '2026-09-24', to: '2026-11-07' })
  })
})

describe('day figures', () => {
  const day = [appointment(1, '2026-10-05', '11:00', '11:50', 'completed'), appointment(2, '2026-10-05', '09:00', '09:45', 'scheduled'), appointment(3, '2026-10-05', '08:00', '09:00', 'cancelled')]

  it('groups by date in start-time order', () => {
    expect(appointmentsByDate(day).get('2026-10-05')?.map((a) => a.id)).toEqual([3, 2, 1])
  })

  it('counts booked minutes without cancelled sessions', () => {
    expect(bookedMinutes(day)).toBe(95)
  })

  it('capitalises the day heading', () => {
    expect(dayHeading('2026-10-05')).toBe('Lunedì 5 ottobre')
  })
})

describe('monthStats', () => {
  it('counts live sessions of the month, held-not-billed and past ones to confirm', () => {
    const list = [
      appointment(1, '2026-10-01', '09:00', '10:00', 'scheduled'),
      appointment(2, '2026-10-02', '09:00', '10:00', 'completed'),
      appointment(3, '2026-10-02', '11:00', '12:00', 'completed', 7),
      appointment(4, '2026-10-20', '09:00', '10:00', 'scheduled'),
      appointment(5, '2026-10-03', '09:00', '10:00', 'cancelled'),
      appointment(6, '2026-09-30', '09:00', '10:00', 'completed'),
    ]
    expect(monthStats(list, 2026, 9, '2026-10-05')).toEqual({ total: 4, heldNotBilled: 1, toConfirm: 1, completed: 2 })
  })
})

describe('withAppointmentStatus', () => {
  it('sends every field back with the new status', () => {
    expect(withAppointmentStatus(appointment(4, '2026-10-01', '09:00', '10:00', 'scheduled'), 'completed')).toEqual({
      id: 4, client_id: 1, service_id: undefined, date: '2026-10-01', start_time: '09:00', end_time: '10:00', status: 'completed', notes: 'n', recurrence_group_id: undefined,
    })
  })
})
