<script setup lang="ts">
/**
 * Writing an invoice. The form on the left, a live summary on the right that
 * recomputes ENPAP, bollo and ritenuta as you type, with the save button next
 * to the total it saves. A line's amount can be typed by hand, replacing
 * quantity × price; quantity visibility starts from the patient, then the profile.
 */
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { Check, ClipboardList, NotebookPen, Plus, Receipt, Trash2, TriangleAlert, UserRound } from 'lucide-vue-next'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import { useConfigStore } from '@/stores/config'
import { useStsStore } from '@/stores/sts'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import FormField from '@/components/ui/FormField.vue'
import ComboBox from '@/components/ui/ComboBox.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import type { ComboOption } from '@/components/ui/types'
import { formatCurrency } from '@/utils/format'
import { hasAmountOverride, lineNetAmount } from '@/utils/tax'
import { useInvoiceTotals } from '@/composables/useInvoiceTotals'
import { useInvoiceForm } from '@/composables/useInvoiceForm'
import { useInvoiceFormLines } from '@/composables/useInvoiceFormLines'
import { useHideQuantityDefault } from '@/composables/useHideQuantityDefault'
import { clientDisplayName } from '@/utils/client'
import { INVOICE_STATUS_OPTIONS as STATUS_OPTIONS, enpapDescription as describeEnpap } from '@/utils/invoiceForm'
import { PAYMENT_METHOD_LABEL, PAYMENT_METHOD_ORDER } from '@/utils/labels'

const route = useRoute()
const clientsStore = useClientsStore()
const servicesStore = useServicesStore()
const configStore = useConfigStore()
const sts = useStsStore()

const editId = route.params.id ? Number(route.params.id) : null
const isEdit = editId !== null
const { form, errors, loading, loadError, saving, nextNumber, nextKey, onSubmit } = useInvoiceForm(editId)
const { addLine, removeLine, applyService, typeAmount, commitAmount, resetAmount } = useInvoiceFormLines(form, nextKey)
const { hideQuantityChosen, hideQuantityOrigin, chooseHideQuantity } = useHideQuantityDefault(form, isEdit)

const clientOptions = computed<ComboOption[]>(() =>
  clientsStore.clients.map((client) => ({ value: client.id, label: clientDisplayName(client), detail: client.fiscal_code })),
)

const taxRegime = computed(() => configStore.config?.tax_regime ?? 'forfettario')
const taxProfile = computed(() => ({
  tax_regime: taxRegime.value,
  enpap_excludes_bollo: configStore.config?.enpap_excludes_bollo ?? false,
}))
const totals = useInvoiceTotals(() => form.lines, () => form.apply_enpap)
const enpapDescription = computed(() => describeEnpap(taxProfile.value))

/** The production Sistema TS holds this invoice: its number, date and paid status are fixed. */
const heldBySts = computed(() => {
  if (editId === null || sts.environment !== 'produzione') return false
  const kind = sts.stateOf(editId).kind
  return kind === 'accepted' || kind === 'sending'
})
</script>

<template>
  <div>
    <PageHeader
      :title="isEdit ? `Modifica fattura N. ${form.invoice_number}` : 'Nuova fattura'"
      :back="isEdit ? { to: `/invoices/${editId}`, label: 'Fattura' } : { to: '/invoices', label: 'Fatture' }"
    />

    <div class="page pt-6 pb-12">
      <div v-if="loading" class="h-96 rounded-card border border-border bg-surface-raised p-6" role="status" aria-busy="true">
        <span class="sr-only">Caricamento della fattura</span>
        <div class="skeleton h-4 w-1/3" />
      </div>

      <EmptyState v-else-if="loadError" :icon="TriangleAlert" title="Non è stato possibile aprire la fattura" :description="loadError">
        <AppButton to="/invoices">Torna alle fatture</AppButton>
      </EmptyState>

      <form v-else class="grid grid-cols-1 items-start gap-5 @min-[72rem]/pane:grid-cols-[minmax(0,1fr)_19rem] @min-[84rem]/pane:grid-cols-[minmax(0,1fr)_22rem]" novalidate @submit.prevent="onSubmit">
        <div class="space-y-5">
          <!-- ── Who and when ─────────────────────────────────────────────── -->
          <AppCard class="settle" :padded="false">
            <CardHeader title="Intestazione" subtitle="A chi, quando e come viene pagata" :icon="UserRound" />
            <div class="grid grid-cols-2 gap-x-4 gap-y-5 px-5 pb-5">
              <FormField v-slot="{ id, invalid, describedBy }" class="col-span-2" label="Paziente" required :error="errors.client">
                <ComboBox
                  :id="id"
                  v-model="form.client_id"
                  :options="clientOptions"
                  placeholder="Cerca per cognome o codice fiscale"
                  :invalid="invalid"
                  :described-by="describedBy"
                />
              </FormField>

              <p v-if="heldBySts" class="col-span-2 rounded-control bg-accent-soft px-3 py-2 text-xs leading-relaxed text-text-muted ring-1 ring-inset ring-accent-line" role="note">
                Registrata al Sistema TS: numero, data e stato pagato restano quelli inviati. Se cambi importi o paziente, dopo il salvataggio invia i dati aggiornati.
              </p>
              <FormField v-slot="{ id, invalid, describedBy }" label="Data di emissione" required :error="errors.issue_date">
                <input :id="id" v-model="form.issue_date" type="date" class="field" :disabled="heldBySts" :aria-invalid="invalid" :aria-describedby="describedBy" />
              </FormField>
              <FormField v-slot="{ id, invalid, describedBy }" label="Scadenza" optional :error="errors.due_date">
                <input :id="id" v-model="form.due_date" type="date" class="field" :aria-invalid="invalid" :aria-describedby="describedBy" />
              </FormField>

              <FormField v-slot="{ id }" label="Metodo di pagamento">
                <select :id="id" v-model="form.payment_method" class="field">
                  <option v-for="method in PAYMENT_METHOD_ORDER" :key="method" :value="method">{{ PAYMENT_METHOD_LABEL[method] }}</option>
                </select>
              </FormField>
              <FormField
                v-if="isEdit"
                v-slot="{ id, invalid, describedBy }"
                label="Numero"
                hint="Cambialo solo per correggere la numerazione: deve restare unico nell'anno."
                :error="errors.invoice_number"
              >
                <input :id="id" v-model="form.invoice_number" type="text" inputmode="numeric" class="field tabular" :disabled="heldBySts" :aria-invalid="invalid" :aria-describedby="describedBy" />
              </FormField>
              <div v-else class="self-end pb-2 text-sm text-text-subtle">
                <template v-if="nextNumber">Riceverà il numero <span class="tabular font-medium text-text-muted">{{ nextNumber }}</span> al salvataggio.</template>
              </div>
            </div>
          </AppCard>

          <!-- ── Lines ──────────────────────────────────────────────────────── -->
          <AppCard class="settle" :padded="false" style="--settle: 1">
            <CardHeader title="Prestazioni" :subtitle="`${form.lines.length} ${form.lines.length === 1 ? 'riga' : 'righe'} in fattura`" :icon="ClipboardList">
              <AppButton size="sm" :icon="Plus" @click="addLine">Aggiungi riga</AppButton>
            </CardHeader>
            <div class="@container/lines">
              <div class="hidden grid-cols-[minmax(0,1fr)_4.5rem_6.5rem_4.5rem_7rem_2rem] gap-2 border-y border-border @min-[44rem]/lines:grid bg-[color-mix(in_srgb,var(--surface-sunken)_55%,var(--surface-raised))] px-5 py-2 text-xs font-medium text-text-subtle" aria-hidden="true">
                <span>Descrizione</span><span class="text-right">Qtà</span><span class="text-right">Prezzo €</span><span class="text-right">IVA %</span><span class="text-right">Importo €</span><span />
              </div>
              <TransitionGroup name="list" tag="div" class="relative border-t border-border @min-[44rem]/lines:border-t-0">
                <div v-for="(line, index) in form.lines" :key="line.key" class="border-b border-border px-5 py-3 last:border-b-0">
                  <div class="grid grid-cols-[repeat(4,minmax(0,1fr))_2rem] items-end gap-2 @min-[44rem]/lines:grid-cols-[minmax(0,1fr)_4.5rem_6.5rem_4.5rem_7rem_2rem] @min-[44rem]/lines:items-start">
                    <div class="col-span-5 space-y-2 @min-[44rem]/lines:col-span-1">
                      <select
                        v-model="line.service_id"
                        class="field field-sm text-text-muted"
                        :aria-label="`Prestazione dal catalogo, riga ${index + 1}`"
                        @change="applyService(line)"
                      >
                        <option :value="undefined">Dal catalogo…</option>
                        <option v-for="service in servicesStore.services" :key="service.id" :value="service.id">{{ service.name }}</option>
                      </select>
                      <input
                        v-model="line.description"
                        type="text"
                        class="field field-sm"
                        placeholder="Descrizione della prestazione"
                        :aria-label="`Descrizione, riga ${index + 1}`"
                        :aria-invalid="errors[`line-${index}`] ? true : undefined"
                      />
                    </div>
                    <label class="block">
                      <span class="mb-1 block text-2xs text-text-subtle @min-[44rem]/lines:sr-only">Qtà</span>
                      <input v-model.number="line.quantity" type="number" min="1" step="1" class="field field-sm tabular text-right" :aria-label="`Quantità, riga ${index + 1}`" />
                    </label>
                    <label class="block">
                      <span class="mb-1 block text-2xs text-text-subtle @min-[44rem]/lines:sr-only">Prezzo €</span>
                      <input v-model.number="line.unit_price" type="number" min="0" step="0.01" class="field field-sm tabular text-right" :aria-label="`Prezzo unitario, riga ${index + 1}`" />
                    </label>
                    <label class="block">
                      <span class="mb-1 block text-2xs text-text-subtle @min-[44rem]/lines:sr-only">IVA %</span>
                      <input v-model.number="line.vat_rate" type="number" min="0" max="100" step="1" class="field field-sm tabular text-right" :aria-label="`Aliquota IVA, riga ${index + 1}`" />
                    </label>
                    <label class="block">
                      <span class="mb-1 block text-2xs text-text-subtle @min-[44rem]/lines:sr-only">Importo €</span>
                      <input
                        :value="hasAmountOverride(line) ? line.amount_override : lineNetAmount(line)"
                        type="number"
                        min="0"
                        step="0.01"
                        class="field field-sm tabular text-right"
                        :class="hasAmountOverride(line) ? 'font-medium text-text' : 'text-text-muted'"
                        :aria-label="`Importo, riga ${index + 1}`"
                        @input="typeAmount(line, $event)"
                        @change="commitAmount(line, $event)"
                      />
                    </label>
                    <AppButton
                      variant="danger-quiet"
                      size="sm"
                      :icon="Trash2"
                      :label="`Rimuovi riga ${index + 1}`"
                      :disabled="form.lines.length === 1"
                      @click="removeLine(line.key)"
                    />
                  </div>
                  <p v-if="hasAmountOverride(line)" class="mt-1.5 text-xs text-text-subtle">
                    Importo scritto a mano al posto di {{ line.quantity }} × {{ formatCurrency(line.unit_price) }}.
                    <button type="button" class="font-medium text-accent underline-offset-2 hover:underline focus-ring" @click="resetAmount(line)">Ricalcola</button>
                  </p>
                  <p v-if="errors[`line-${index}`]" class="mt-1.5 text-xs text-danger" role="alert">{{ errors[`line-${index}`] }}</p>
                </div>
              </TransitionGroup>
            </div>
          </AppCard>

          <!-- ── Status and notes ─────────────────────────────────────────── -->
          <AppCard class="settle" :padded="false" style="--settle: 2">
            <CardHeader title="Stato e note" subtitle="Come la fattura esce dallo studio" :icon="NotebookPen" tone="neutral" />
            <div class="space-y-5 px-5 pb-5">
              <div>
                <p class="mb-1.5 text-sm font-medium text-text-muted">Stato</p>
                <SegmentedControl v-model="form.status" :options="STATUS_OPTIONS" label="Stato della fattura" />
              </div>
              <FormField v-if="form.status === 'paid'" v-slot="{ id, describedBy }" label="Pagata il" hint="Se la lasci vuota vale la data di oggi." class="max-w-60">
                <input :id="id" v-model="form.paid_date" type="date" class="field" :aria-describedby="describedBy" />
              </FormField>
              <ToggleSwitch v-model="form.apply_enpap" label="Contributo integrativo ENPAP 2%" :description="enpapDescription" />
              <ToggleSwitch
                :model-value="form.hide_quantity"
                label="Nascondi quantità e prezzo unitario"
                :description="`In fattura resta solo l'importo per riga. ${hideQuantityChosen ? 'Scelto per questa fattura.' : hideQuantityOrigin}`"
                @update:model-value="chooseHideQuantity"
              />
              <FormField v-slot="{ id }" label="Note" optional>
                <textarea :id="id" v-model="form.notes" rows="2" class="field" placeholder="Compaiono in fondo alla fattura" />
              </FormField>
            </div>
          </AppCard>
        </div>

        <!-- ── Live summary and save ─────────────────────────────────────── -->
        <aside class="settle @min-[72rem]/pane:sticky @min-[72rem]/pane:top-30" style="--settle: 1">
          <AppCard :padded="false">
            <CardHeader title="Riepilogo" subtitle="Si aggiorna mentre scrivi" :icon="Receipt" />
            <div class="px-5 pb-5">
              <dl class="space-y-2 text-base">
                <div class="flex justify-between"><dt class="text-text-muted">Imponibile</dt><dd class="tabular text-text">{{ formatCurrency(totals.total_net) }}</dd></div>
                <div v-if="totals.total_tax > 0" class="flex justify-between"><dt class="text-text-muted">IVA</dt><dd class="tabular text-text">{{ formatCurrency(totals.total_tax) }}</dd></div>
                <div v-if="totals.marca_da_bollo > 0" class="flex justify-between"><dt class="text-text-muted">Marca da bollo</dt><dd class="tabular text-text">{{ formatCurrency(totals.marca_da_bollo) }}</dd></div>
                <div v-if="form.apply_enpap" class="flex justify-between"><dt class="text-text-muted">ENPAP 2%</dt><dd class="tabular text-text">{{ formatCurrency(totals.contributo_enpap) }}</dd></div>
                <div v-if="totals.ritenuta_acconto > 0" class="flex justify-between"><dt class="text-text-muted">Ritenuta 20%</dt><dd class="tabular text-text">−{{ formatCurrency(totals.ritenuta_acconto) }}</dd></div>
              </dl>
              <div class="receipt-total -mx-5 mt-4 px-5 py-4">
                <p class="label-quiet">Totale dovuto</p>
                <p class="tabular mt-0.5 text-[2rem] font-semibold leading-tight tracking-[-0.02em] text-text" aria-live="polite">{{ formatCurrency(totals.total_due) }}</p>
              </div>
              <AppButton type="submit" variant="primary" class="mt-5" block :icon="Check" :loading="saving">
                {{ isEdit ? 'Salva modifiche' : 'Crea fattura' }}
              </AppButton>
              <p v-if="Object.keys(errors).length > 0" class="mt-3 text-center text-xs text-danger" role="alert">Controlla i campi segnalati.</p>
            </div>
          </AppCard>
          <p class="mt-3 px-1 text-xs text-text-subtle">
            {{ taxRegime === 'forfettario' ? 'Regime forfettario: niente IVA né ritenuta; bollo da 2 € sopra i 77,47 €.' : 'Regime ordinario: la ritenuta d’acconto del 20% è calcolata in automatico.' }}
          </p>
        </aside>
      </form>
    </div>
  </div>
</template>

<style scoped>
/* The total sits on a strip ruled top and bottom with a dotted line, like the foot of a receipt. */
.receipt-total {
  background: color-mix(in srgb, var(--surface-sunken) 45%, transparent);
  border-block: 1px dashed var(--border-strong);
}
</style>
