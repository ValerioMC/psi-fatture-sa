/** Presentation rules for patients (clients), shared by lists, pickers and the PDF. */
import type { Client } from '@/types'

/**
 * How a patient is named in lists: surname first for people, so an alphabetical
 * list reads the way a paper register does; the business name for companies.
 */
export function clientDisplayName(client: Pick<Client, 'client_type' | 'first_name' | 'last_name'>): string {
  if (client.client_type === 'azienda') return client.last_name.trim() || client.first_name.trim()
  return `${client.last_name} ${client.first_name}`.trim()
}

/** Whole years of age on a given day, or null without a birth date. */
export function ageOn(birthDate: string | undefined, today: Date = new Date()): number | null {
  if (!birthDate) return null
  const [year, month, day] = birthDate.slice(0, 10).split('-').map(Number)
  if (!year || !month || !day) return null
  let age = today.getFullYear() - year
  const beforeBirthday = today.getMonth() + 1 < month || (today.getMonth() + 1 === month && today.getDate() < day)
  if (beforeBirthday) age -= 1
  return age >= 0 ? age : null
}
