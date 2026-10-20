/** Adding, removing and filling the lines of the invoice form. */
import { useServicesStore } from '@/features/service/store/services'
import { blankLine, type FormLine, type InvoiceForm } from '@/features/invoice/lib/invoiceForm'

export function useInvoiceFormLines(form: InvoiceForm, nextKey: () => number) {
  const servicesStore = useServicesStore()

  function addLine(): void {
    form.lines.push(blankLine(nextKey()))
  }

  function removeLine(key: number): void {
    form.lines = form.lines.filter((line) => line.key !== key)
  }

  /** Copies the chosen catalogue service's name, price and VAT into the line. */
  function applyService(line: FormLine): void {
    const service = servicesStore.services.find((candidate) => candidate.id === line.service_id)
    if (service === undefined) return
    line.description = service.name
    line.unit_price = service.default_price
    line.vat_rate = service.vat_rate
  }

  /** Live while typing; an emptied field waits for `commitAmount` on change. */
  function typeAmount(line: FormLine, event: Event): void {
    const amount = (event.target as HTMLInputElement).valueAsNumber
    if (Number.isFinite(amount)) line.amount_override = amount
  }

  function commitAmount(line: FormLine, event: Event): void {
    if ((event.target as HTMLInputElement).value.trim() === '') resetAmount(line)
  }

  /** Back to quantity × unit price. */
  function resetAmount(line: FormLine): void {
    line.amount_override = null
  }

  return { addLine, removeLine, applyService, typeAmount, commitAmount, resetAmount }
}
