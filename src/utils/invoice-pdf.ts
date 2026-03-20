import { jsPDF } from 'jspdf'
import type { Invoice, Client, ProfessionalConfig } from '@/types'
import { formatCurrency, formatDateLong } from './format'

const PAGE_W = 210
const MARGIN = 15
const CONTENT_W = PAGE_W - 2 * MARGIN
const RIGHT = PAGE_W - MARGIN

/** Sets font size, weight and text color on the PDF. */
function style(
  pdf: jsPDF,
  size: number,
  bold = false,
  color: [number, number, number] = [30, 30, 30],
): void {
  pdf.setFontSize(size)
  pdf.setFont('helvetica', bold ? 'bold' : 'normal')
  pdf.setTextColor(color[0], color[1], color[2])
}

function renderHeader(pdf: jsPDF, config: ProfessionalConfig, invoice: Invoice, y: number): number {
  const fullName = [config.title, config.first_name, config.last_name].filter(Boolean).join(' ')
  style(pdf, 14, true)
  pdf.text(fullName, MARGIN, y)
  y += 6

  const lines = [
    `P.IVA: ${config.vat_number}  C.F.: ${config.fiscal_code}`,
    `${config.address}, ${config.zip_code} ${config.city} (${config.province})`,
    config.phone && `Tel: ${config.phone}`,
    config.pec_email && `PEC: ${config.pec_email}`,
    config.albo_number && `Albo: n. ${config.albo_number} – ${config.albo_region}`,
  ].filter(Boolean) as string[]

  style(pdf, 9, false, [100, 100, 100])
  for (const line of lines) {
    pdf.text(line, MARGIN, y)
    y += 4.5
  }

  renderInvoiceBox(pdf, invoice)
  return Math.max(y, MARGIN + 33)
}

function renderInvoiceBox(pdf: jsPDF, invoice: Invoice): void {
  const x = PAGE_W - MARGIN - 45
  pdf.setDrawColor(30, 30, 30)
  pdf.setLineWidth(0.5)
  pdf.rect(x, MARGIN, 45, 18)
  style(pdf, 7, true, [100, 100, 100])
  pdf.text('FATTURA', x + 22.5, MARGIN + 5.5, { align: 'center' })
  style(pdf, 16, true)
  pdf.text(`${invoice.invoice_number}/${invoice.year}`, x + 22.5, MARGIN + 13, { align: 'center' })
  style(pdf, 9, false, [100, 100, 100])
  pdf.text(`Data: ${formatDateLong(invoice.issue_date)}`, RIGHT, MARGIN + 22, { align: 'right' })
  if (invoice.due_date) {
    pdf.text(`Scadenza: ${formatDateLong(invoice.due_date)}`, RIGHT, MARGIN + 27, { align: 'right' })
  }
}

function renderClientBlock(pdf: jsPDF, client: Client, y: number): number {
  style(pdf, 7, false, [150, 150, 150])
  pdf.text('Spett.le', MARGIN, y + 8)
  y += 12

  pdf.setFillColor(248, 248, 248)
  pdf.rect(MARGIN, y - 2, CONTENT_W, 26, 'F')
  style(pdf, 11, true)
  pdf.text(`${client.first_name} ${client.last_name}`, MARGIN + 4, y + 4)

  const details = [
    client.fiscal_code && `C.F.: ${client.fiscal_code}`,
    client.vat_number && `P.IVA: ${client.vat_number}`,
    `${client.address}, ${client.zip_code} ${client.city} (${client.province})`,
    client.email,
  ].filter(Boolean) as string[]

  style(pdf, 9, false, [100, 100, 100])
  let dy = y + 10
  for (const line of details) {
    pdf.text(line, MARGIN + 4, dy)
    dy += 4.5
  }
  return y + 29
}

function renderTableHeader(pdf: jsPDF, y: number): void {
  pdf.setFillColor(30, 30, 30)
  pdf.rect(MARGIN, y, CONTENT_W, 7, 'F')
  style(pdf, 8, true, [255, 255, 255])
  pdf.text('Descrizione', MARGIN + 2, y + 4.5)
  pdf.text('Qtà', MARGIN + CONTENT_W * 0.62, y + 4.5, { align: 'center' })
  pdf.text('Prezzo unit.', MARGIN + CONTENT_W * 0.77, y + 4.5, { align: 'right' })
  pdf.text('IVA%', MARGIN + CONTENT_W * 0.88, y + 4.5, { align: 'right' })
  pdf.text('Totale', RIGHT - 2, y + 4.5, { align: 'right' })
}

function renderItemsTable(pdf: jsPDF, invoice: Invoice, y: number): number {
  y += 4
  renderTableHeader(pdf, y)
  y += 7

  for (const line of invoice.lines) {
    pdf.setFillColor(252, 252, 252)
    pdf.rect(MARGIN, y, CONTENT_W, 7, 'F')
    pdf.setDrawColor(235, 235, 235)
    pdf.line(MARGIN, y + 7, RIGHT, y + 7)
    style(pdf, 8.5, false)
    pdf.text(line.description, MARGIN + 2, y + 4.5)
    style(pdf, 8.5, false, [100, 100, 100])
    pdf.text(String(line.quantity), MARGIN + CONTENT_W * 0.62, y + 4.5, { align: 'center' })
    pdf.text(formatCurrency(line.unit_price), MARGIN + CONTENT_W * 0.77, y + 4.5, { align: 'right' })
    pdf.text(`${line.vat_rate}%`, MARGIN + CONTENT_W * 0.88, y + 4.5, { align: 'right' })
    style(pdf, 8.5, true)
    pdf.text(formatCurrency(line.line_total), RIGHT - 2, y + 4.5, { align: 'right' })
    y += 7
  }
  return y
}

function renderTotalsRow(
  pdf: jsPDF,
  label: string,
  value: string,
  y: number,
  bold: boolean,
  valueColor: [number, number, number],
): void {
  const totalsX = PAGE_W - MARGIN - 60
  style(pdf, bold ? 10 : 9, bold, [100, 100, 100])
  pdf.text(label, totalsX, y)
  style(pdf, bold ? 10 : 9, bold, valueColor)
  pdf.text(value, RIGHT, y, { align: 'right' })
}

function renderTotals(pdf: jsPDF, invoice: Invoice, y: number): number {
  const totalsX = PAGE_W - MARGIN - 60
  y += 8

  renderTotalsRow(pdf, 'Totale netto', formatCurrency(invoice.total_net), y, false, [100, 100, 100])
  y += 5
  if (invoice.total_tax > 0) {
    renderTotalsRow(pdf, 'IVA', formatCurrency(invoice.total_tax), y, false, [100, 100, 100])
    y += 5
  }
  if (invoice.apply_enpap && invoice.contributo_enpap > 0) {
    renderTotalsRow(pdf, 'Contributo ENPAP (2%)', formatCurrency(invoice.contributo_enpap), y, false, [100, 100, 100])
    y += 5
  }
  pdf.setDrawColor(150, 150, 150)
  pdf.line(totalsX, y, RIGHT, y)
  y += 3
  renderTotalsRow(pdf, 'Totale lordo', formatCurrency(invoice.total_gross), y, false, [100, 100, 100])
  y += 5
  if (invoice.ritenuta_acconto > 0) {
    renderTotalsRow(pdf, "Ritenuta d'acconto (20%)", `– ${formatCurrency(invoice.ritenuta_acconto)}`, y, false, [200, 50, 50])
    y += 5
  }
  if (invoice.marca_da_bollo) {
    renderTotalsRow(pdf, 'Marca da bollo', `+ ${formatCurrency(2)}`, y, false, [100, 100, 100])
    y += 5
  }
  pdf.setDrawColor(30, 30, 30)
  pdf.setLineWidth(0.6)
  pdf.line(totalsX, y, RIGHT, y)
  y += 3
  renderTotalsRow(pdf, 'Totale dovuto', formatCurrency(invoice.total_due), y, true, [30, 30, 30])
  return y + 6
}

function renderFooter(
  pdf: jsPDF,
  invoice: Invoice,
  config: ProfessionalConfig,
  noteLines: string[],
  y: number,
): number {
  y += 4
  if (config.iban) {
    pdf.setFillColor(248, 248, 248)
    pdf.rect(MARGIN, y - 2, CONTENT_W, 10, 'F')
    style(pdf, 9, true, [50, 50, 50])
    pdf.text('Pagamento:', MARGIN + 4, y + 4)
    style(pdf, 9, false, [100, 100, 100])
    pdf.text(config.iban, MARGIN + 30, y + 4)
    y += 13
  }

  if (noteLines.length > 0) {
    pdf.setDrawColor(220, 220, 220)
    pdf.line(MARGIN, y, RIGHT, y)
    y += 5
    for (const note of noteLines) {
      style(pdf, 7.5, false, [120, 120, 120])
      const wrapped = pdf.splitTextToSize(note, CONTENT_W) as string[]
      pdf.text(wrapped, MARGIN, y)
      y += wrapped.length * 4 + 2
    }
  }

  if (invoice.notes) {
    y += 3
    style(pdf, 9, true, [50, 50, 50])
    pdf.text('Note:', MARGIN, y)
    style(pdf, 9, false, [100, 100, 100])
    const wrapped = pdf.splitTextToSize(invoice.notes, CONTENT_W - 16) as string[]
    pdf.text(wrapped, MARGIN + 16, y)
  }

  return y
}

/** Builds a jsPDF invoice document from the given data. */
export function buildInvoicePdf(
  invoice: Invoice,
  client: Client,
  config: ProfessionalConfig,
  noteLines: string[],
): jsPDF {
  const pdf = new jsPDF({ unit: 'mm', format: 'a4', orientation: 'portrait' })
  let y = MARGIN

  y = renderHeader(pdf, config, invoice, y)
  pdf.setDrawColor(200, 200, 200)
  pdf.setLineWidth(0.3)
  pdf.line(MARGIN, y, RIGHT, y)
  y = renderClientBlock(pdf, client, y)
  y = renderItemsTable(pdf, invoice, y)
  y = renderTotals(pdf, invoice, y)
  renderFooter(pdf, invoice, config, noteLines, y)

  return pdf
}
