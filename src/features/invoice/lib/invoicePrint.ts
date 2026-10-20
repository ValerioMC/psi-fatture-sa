/** What the printable invoice says, apart from how it is laid out. */
import type { Client } from '@/features/client/model/client'
import type { Invoice, PaymentMethod } from '@/features/invoice/model/invoice'
import type { ProfessionalConfig } from '@/features/settings/model/config'
import { formatCurrency } from '@/common/lib/format'

const FORFETTARIO_NOTE =
  "Operazione senza applicazione dell'IVA effettuata ai sensi dell'art. 1, commi da 54 a 89, L. n. 190 del 2014 e modificato dalla L.n. 208 del 2015 e dalla L.n. 145 del 2018 – Regime Forfettario. Imposta non dovuta."
const EXEMPT_NOTE = "Operazione esente da IVA ai sensi dell'art. 10, n. 18, DPR 633/72."

/** The printed form, longer than the picker label in `labels.ts` for card payments. */
const PRINTED_PAYMENT_LABEL: Readonly<Record<PaymentMethod, string>> = {
  bonifico: 'Bonifico bancario',
  contanti: 'Contanti',
  pos: 'POS / Carta di credito',
  altro: 'Altro',
}

/** Cash and card are settled at the session; "other" states no condition. */
const PAYMENT_CONDITION: Readonly<Partial<Record<PaymentMethod, string>>> = {
  bonifico: 'Pagamento a vista',
  contanti: 'Contestuale alla prestazione',
  pos: 'Contestuale alla prestazione',
}

export function printedPaymentLabel(method: PaymentMethod): string {
  return PRINTED_PAYMENT_LABEL[method] ?? method
}

export function paymentCondition(method: PaymentMethod): string | undefined {
  return PAYMENT_CONDITION[method]
}

export function professionalFullName(person: Pick<ProfessionalConfig, 'title' | 'first_name' | 'last_name'>): string {
  return [person.title, person.first_name, person.last_name].filter(Boolean).join(' ')
}

/** The patient as the invoice addresses them: first name first, the business name for companies. */
export function printedClientName(client: Pick<Client, 'client_type' | 'first_name' | 'last_name'>): string {
  if (client.client_type === 'azienda') return client.last_name
  return `${client.first_name} ${client.last_name}`.trim()
}

export function hasVat(invoice: Pick<Invoice, 'total_tax'>): boolean {
  return invoice.total_tax > 0
}

/**
 * The legal sentences under the totals. The forfettario and ENPAP ones share a paragraph
 * when both apply; the VAT-exempt one is for the ordinary regime only.
 */
export function legalNotes(
  invoice: Pick<Invoice, 'apply_enpap' | 'contributo_enpap' | 'ritenuta_acconto' | 'total_tax'>,
  config: Pick<ProfessionalConfig, 'tax_regime'>,
): string[] {
  const isForfettario = config.tax_regime === 'forfettario'
  const enpapNote =
    invoice.apply_enpap && invoice.contributo_enpap > 0
      ? `Contributo integrativo ENPAP 2% (${formatCurrency(invoice.contributo_enpap)}) addebitato al cliente ai sensi dell'art. 8, L. 21/86.`
      : ''
  const notes = isForfettario ? joinParagraph(FORFETTARIO_NOTE, enpapNote) : joinParagraph('', enpapNote)

  if (!hasVat(invoice) && !isForfettario) notes.push(EXEMPT_NOTE)
  if (invoice.ritenuta_acconto > 0) {
    notes.push(`Si richiede di operare una ritenuta d'acconto del 20% pari a ${formatCurrency(invoice.ritenuta_acconto)}.`)
  }
  return notes
}

function joinParagraph(first: string, second: string): string[] {
  if (first && second) return [`${first} ${second}`]
  return [first, second].filter(Boolean)
}
