/** How the professional's own name reads on the letterhead and the printed invoice. */
import type { ProfessionalConfig } from '@/features/settings/model/config'

export function professionalFullName(person: Pick<ProfessionalConfig, 'title' | 'first_name' | 'last_name'>): string {
  return [person.title, person.first_name, person.last_name].filter(Boolean).join(' ')
}
