<script setup lang="ts">
/**
 * One invoice: the document on the left, its life and its actions on the
 * right. The next step is always the primary button: a draft is issued, an
 * issued or overdue invoice is marked paid.
 */
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { CircleCheck, Pencil, Printer, Send, Trash2, TriangleAlert } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import { errorMessage, useToastStore } from '@/stores/toast'
import type { Invoice, InvoiceStatus } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import InvoiceSeal from '@/components/ui/InvoiceSeal.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { formatCurrency, formatDateLong, todayIso } from '@/utils/format'
import { PAYMENT_METHOD_LABEL } from '@/utils/labels'
import { deriveSeal, describeSeal } from '@/utils/invoiceSeal'
import { withStatus } from '@/utils/invoiceInput'

const route = useRoute()
const router = useRouter()
const invoicesStore = useInvoicesStore()
const toast = useToastStore()

const invoiceId = Number(route.params.id)
const invoice = ref<Invoice | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)
const updating = ref(false)
const deleteOpen = ref(false)
const deleting = ref(false)

onMounted(async () => {
  try {
    invoice.value = await invoicesStore.fetchInvoice(invoiceId)
  } catch (error) {
    loadError.value = errorMessage(error)
  } finally {
    loading.value = false
  }
})

const seal = computed(() => (invoice.value ? deriveSeal(invoice.value) : null))

const sealSentence = computed(() => {
  if (invoice.value === null || seal.value === null) return ''
  if (invoice.value.status === 'paid') return invoice.value.paid_date ? `Incassata il ${formatDateLong(invoice.value.paid_date)}` : ''
  if (invoice.value.status === 'cancelled') return 'Non ha valore fiscale'
  return describeSeal(seal.value)
})

const nextStep = computed<{ status: InvoiceStatus; label: string } | null>(() => {
  const status = invoice.value?.status
  if (status === 'draft') return { status: 'issued', label: 'Emetti fattura' }
  if (status === 'issued' || status === 'overdue') return { status: 'paid', label: 'Segna come pagata' }
  return null
})

async function changeStatus(status: InvoiceStatus, announce = true): Promise<void> {
  const current = invoice.value
  if (current === null) return
  const previous = current.status
  updating.value = true
  try {
    invoice.value = await invoicesStore.editInvoice(withStatus(current, status, status === 'paid' ? todayIso() : undefined))
    if (!announce) return
    const message = status === 'paid' ? 'Fattura segnata come pagata' : status === 'issued' ? 'Fattura emessa' : 'Stato aggiornato'
    toast.notify(message, { label: 'Annulla', run: () => changeStatus(previous, false) })
  } catch (error) {
    toast.notifyError(error, 'Aggiornamento non riuscito')
  } finally {
    updating.value = false
  }
}

async function confirmDelete(): Promise<void> {
  if (invoice.value === null) return
  deleting.value = true
  try {
    await invoicesStore.removeInvoice(invoice.value.id)
    toast.notify(`Fattura N. ${invoice.value.invoice_number} eliminata`)
    void router.push('/invoices')
  } catch (error) {
    toast.notifyError(error, 'Eliminazione non riuscita')
    deleting.value = false
  }
}

interface Milestone {
  label: string
  date: string | null
  state: 'done' | 'due' | 'late' | 'pending'
}

/** The invoice's life as dated steps: issued, due, paid. */
const milestones = computed<Milestone[]>(() => {
  const current = invoice.value
  if (current === null || current.status === 'draft' || current.status === 'cancelled') return []
  const list: Milestone[] = [{ label: 'Emessa', date: current.issue_date, state: 'done' }]
  if (current.due_date) {
    const late = seal.value?.kind === 'overdue'
    list.push({ label: late ? 'Scaduta' : 'Scadenza', date: current.due_date, state: current.status === 'paid' ? 'done' : late ? 'late' : 'due' })
  }
  list.push({ label: 'Pagata', date: current.paid_date ?? null, state: current.status === 'paid' ? 'done' : 'pending' })
  return list
})

const MILESTONE_DOT: Record<Milestone['state'], string> = {
  done: 'bg-safe ring-safe-line',
  due: 'bg-warn ring-warn-line',
  late: 'bg-danger ring-danger-line',
  pending: 'bg-surface-raised ring-border-strong',
}
</script>

<template>
  <div>
    <PageHeader
      :title="invoice ? `Fattura N. ${invoice.invoice_number}/${invoice.year}` : 'Fattura'"
      :back="{ to: '/invoices', label: 'Fatture' }"
    >
      <template v-if="invoice">
        <AppButton :icon="Printer" :to="`/invoices/${invoice.id}/print`">Stampa o PDF</AppButton>
        <AppButton :icon="Pencil" :to="`/invoices/${invoice.id}/edit`">Modifica</AppButton>
      </template>
    </PageHeader>

    <div class="page pt-6 pb-12">
      <div v-if="loading" class="grid grid-cols-[1fr_18rem] gap-5" role="status" aria-busy="true">
        <span class="sr-only">Caricamento della fattura</span>
        <div class="h-[28rem] rounded-card border border-border bg-surface-raised p-8"><div class="skeleton h-4 w-1/3" /></div>
        <div class="h-64 rounded-card border border-border bg-surface-raised p-5"><div class="skeleton h-4 w-1/2" /></div>
      </div>

      <EmptyState v-else-if="loadError || !invoice" :icon="TriangleAlert" title="Fattura non trovata" :description="loadError ?? undefined">
        <AppButton to="/invoices">Torna alle fatture</AppButton>
      </EmptyState>

      <div v-else class="grid grid-cols-1 items-start gap-5 lg:grid-cols-[minmax(0,1fr)_18rem] 2xl:grid-cols-[minmax(0,1fr)_21rem]">
        <!-- ── The document ─────────────────────────────────────────────────── -->
        <AppCard as="article" :padded="false" class="settle overflow-hidden">
          <header class="flex items-start justify-between gap-6 px-8 pt-7 pb-6">
            <div class="flex min-w-0 items-center gap-3.5">
              <PatientMonogram :name="invoice.client_name" size="lg" />
              <div class="min-w-0">
                <p class="label-quiet">Intestata a</p>
                <RouterLink :to="`/clients/${invoice.client_id}/edit`" class="display block truncate text-2xl text-text hover:underline decoration-border-strong underline-offset-4">
                  {{ invoice.client_name }}
                </RouterLink>
              </div>
            </div>
            <dl class="grid shrink-0 grid-cols-[auto_auto] gap-x-5 gap-y-1 text-sm">
              <dt class="text-text-subtle">Emessa il</dt>
              <dd class="text-right text-text">{{ formatDateLong(invoice.issue_date) }}</dd>
              <template v-if="invoice.due_date">
                <dt class="text-text-subtle">Scadenza</dt>
                <dd class="text-right" :class="seal?.kind === 'overdue' ? 'font-medium text-danger' : 'text-text'">{{ formatDateLong(invoice.due_date) }}</dd>
              </template>
              <dt class="text-text-subtle">Pagamento</dt>
              <dd class="text-right text-text">{{ PAYMENT_METHOD_LABEL[invoice.payment_method] }}</dd>
            </dl>
          </header>

          <table class="w-full text-base">
            <thead class="border-y border-border bg-surface text-left text-xs text-text-subtle">
              <tr class="h-9">
                <th class="pl-8 font-medium">Prestazione</th>
                <th class="w-16 text-right font-medium">Qtà</th>
                <th class="w-32 text-right font-medium">Prezzo</th>
                <th class="w-20 text-right font-medium">IVA</th>
                <th class="w-32 pr-8 text-right font-medium">Importo</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(line, index) in invoice.lines" :key="line.id ?? index" class="border-b border-border">
                <td class="py-3 pl-8 text-text">{{ line.description }}</td>
                <td class="tabular text-right text-text-muted">{{ line.quantity }}</td>
                <td class="tabular text-right text-text-muted">{{ formatCurrency(line.unit_price) }}</td>
                <td class="text-right text-sm text-text-subtle">{{ line.vat_rate > 0 ? `${line.vat_rate}%` : 'esente' }}</td>
                <td class="tabular pr-8 text-right font-medium text-text">{{ formatCurrency(line.line_total) }}</td>
              </tr>
            </tbody>
          </table>

          <div class="flex justify-end px-8 py-6">
            <dl class="w-80 space-y-2 text-base">
              <div class="flex justify-between"><dt class="text-text-muted">Imponibile</dt><dd class="tabular text-text">{{ formatCurrency(invoice.total_net) }}</dd></div>
              <div v-if="invoice.total_tax > 0" class="flex justify-between"><dt class="text-text-muted">IVA</dt><dd class="tabular text-text">{{ formatCurrency(invoice.total_tax) }}</dd></div>
              <div v-if="invoice.apply_enpap && invoice.contributo_enpap > 0" class="flex justify-between"><dt class="text-text-muted">Contributo ENPAP 2%</dt><dd class="tabular text-text">{{ formatCurrency(invoice.contributo_enpap) }}</dd></div>
              <div v-if="invoice.marca_da_bollo" class="flex justify-between"><dt class="text-text-muted">Marca da bollo</dt><dd class="tabular text-text">{{ formatCurrency(2) }}</dd></div>
              <div v-if="invoice.ritenuta_acconto > 0" class="flex justify-between"><dt class="text-text-muted">Ritenuta d'acconto 20%</dt><dd class="tabular text-text">−{{ formatCurrency(invoice.ritenuta_acconto) }}</dd></div>
              <div class="flex items-baseline justify-between border-t border-border pt-3">
                <dt class="font-medium text-text">Totale dovuto</dt>
                <dd class="text-2xl font-semibold tracking-[-0.01em] text-text">{{ formatCurrency(invoice.total_due) }}</dd>
              </div>
            </dl>
          </div>

          <footer v-if="invoice.notes" class="border-t border-border bg-surface px-8 py-4">
            <p class="label-quiet">Note</p>
            <p class="mt-1 whitespace-pre-wrap text-base text-text-muted">{{ invoice.notes }}</p>
          </footer>
        </AppCard>

        <!-- ── Its life and what to do next ─────────────────────────────────── -->
        <aside class="settle space-y-4 lg:sticky lg:top-30" style="--settle: 1">
          <AppCard>
            <div class="flex items-center gap-3.5">
              <InvoiceSeal :status="invoice.status" :issue-date="invoice.issue_date" :due-date="invoice.due_date" :size="40" live />
              <div class="min-w-0">
                <StatusBadge type="invoice" :status="invoice.status" />
                <p v-if="sealSentence" class="mt-1 text-sm text-text-muted">{{ sealSentence }}</p>
              </div>
            </div>

            <ol v-if="milestones.length > 0" class="relative mt-5 space-y-3.5 border-t border-border pt-5">
              <li v-for="milestone in milestones" :key="milestone.label" class="relative flex items-center gap-3 text-sm">
                <span class="size-2.5 shrink-0 rounded-full ring-[3px]" :class="MILESTONE_DOT[milestone.state]" aria-hidden="true" />
                <span class="flex-1" :class="milestone.state === 'pending' ? 'text-text-subtle' : 'text-text'">{{ milestone.label }}</span>
                <span class="tabular" :class="milestone.state === 'late' ? 'text-danger' : 'text-text-subtle'">{{ milestone.date ? formatDateLong(milestone.date) : '—' }}</span>
              </li>
            </ol>

            <AppButton
              v-if="nextStep"
              class="mt-5"
              variant="primary"
              block
              :icon="nextStep.status === 'paid' ? CircleCheck : Send"
              :loading="updating"
              @click="changeStatus(nextStep.status)"
            >
              {{ nextStep.label }}
            </AppButton>
          </AppCard>

          <AppButton variant="danger-quiet" :icon="Trash2" block @click="deleteOpen = true">Elimina fattura</AppButton>
        </aside>
      </div>
    </div>

    <ConfirmDialog
      :open="deleteOpen"
      title="Eliminare la fattura?"
      message="L'operazione non si può annullare."
      :blast-radius="invoice ? `La fattura N. ${invoice.invoice_number}/${invoice.year} di ${invoice.client_name}, ${formatCurrency(invoice.total_due)}, verrà eliminata definitivamente.` : undefined"
      :loading="deleting"
      @confirm="confirmDelete"
      @cancel="deleteOpen = false"
    />
  </div>
</template>
