<script setup lang="ts">
/**
 * End-of-month invoicing: every session held and not yet invoiced becomes one
 * invoice per patient. Pick the month, untick who should wait, generate. The
 * preview loads as soon as the month changes; there is no "load" step.
 */
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { CalendarCheck, FileText, Sparkles } from 'lucide-vue-next'
import { generateMonthlyInvoices, previewMonthlyInvoices } from '@/api'
import type { MonthlyInvoicePreview, PaymentMethod } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import PeriodStepper from '@/components/ui/PeriodStepper.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import FormField from '@/components/ui/FormField.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { useToastStore } from '@/stores/toast'
import { formatCurrency, formatMonthYear } from '@/utils/format'
import { PAYMENT_METHOD_LABEL, PAYMENT_METHOD_ORDER, plural } from '@/utils/labels'

const route = useRoute()
const toast = useToastStore()

const now = new Date()
/** Default to last month: invoices are written once a month has closed. */
const defaultMonth = now.getMonth() === 0 ? { year: now.getFullYear() - 1, month: 12 } : { year: now.getFullYear(), month: now.getMonth() }
const queryYear = Number(route.query.year)
const queryMonth = Number(route.query.month)
const year = ref(Number.isInteger(queryYear) && queryYear > 2000 ? queryYear : defaultMonth.year)
const month = ref(Number.isInteger(queryMonth) && queryMonth >= 1 && queryMonth <= 12 ? queryMonth : defaultMonth.month)

const paymentMethod = ref<PaymentMethod>('bonifico')
const applyEnpap = ref(true)
const previews = ref<MonthlyInvoicePreview[]>([])
const selected = ref<Set<number>>(new Set())
const loading = ref(false)
const generating = ref(false)
const confirmOpen = ref(false)
const generated = ref<number | null>(null)

const label = computed(() => formatMonthYear(year.value, month.value))
const isFutureMonth = computed(() => year.value > now.getFullYear() || (year.value === now.getFullYear() && month.value > now.getMonth() + 1))

function shift(delta: number): void {
  let nextMonth = month.value + delta
  let nextYear = year.value
  if (nextMonth < 1) { nextMonth = 12; nextYear-- }
  if (nextMonth > 12) { nextMonth = 1; nextYear++ }
  month.value = nextMonth
  year.value = nextYear
}

async function loadPreview(): Promise<void> {
  loading.value = true
  generated.value = null
  try {
    previews.value = await previewMonthlyInvoices(year.value, month.value)
    selected.value = new Set(previews.value.map((preview) => preview.client_id))
  } catch (error) {
    previews.value = []
    toast.notifyError(error, 'Anteprima non disponibile')
  } finally {
    loading.value = false
  }
}

watch([year, month], loadPreview)
onMounted(loadPreview)

const chosen = computed(() => previews.value.filter((preview) => selected.value.has(preview.client_id)))
const totals = computed(() => ({
  sessions: chosen.value.reduce((sum, preview) => sum + preview.appointment_count, 0),
  due: chosen.value.reduce((sum, preview) => sum + preview.estimated_due, 0),
}))
const allSelected = computed(() => previews.value.length > 0 && selected.value.size === previews.value.length)

function toggle(clientId: number): void {
  const next = new Set(selected.value)
  if (next.has(clientId)) next.delete(clientId)
  else next.add(clientId)
  selected.value = next
}

function toggleAll(): void {
  selected.value = allSelected.value ? new Set() : new Set(previews.value.map((preview) => preview.client_id))
}

async function generate(): Promise<void> {
  generating.value = true
  try {
    const result = await generateMonthlyInvoices({
      year: year.value,
      month: month.value,
      client_ids: [...selected.value],
      payment_method: paymentMethod.value,
      apply_enpap: applyEnpap.value,
    })
    confirmOpen.value = false
    generated.value = result.length
    previews.value = []
  } catch (error) {
    toast.notifyError(error, 'Generazione non riuscita')
  } finally {
    generating.value = false
  }
}
</script>

<template>
  <div>
    <PageHeader title="Fatturazione mensile" :back="{ to: '/invoices', label: 'Fatture' }">
      <PeriodStepper
        :label="label"
        previous-label="Mese precedente"
        next-label="Mese successivo"
        :can-next="!isFutureMonth"
        min-width="9.5rem"
        @previous="shift(-1)"
        @next="shift(1)"
      />
    </PageHeader>

    <div class="mx-auto max-w-[72rem] px-8 pt-6 pb-12">
      <!-- Done: the stamp lands once. -->
      <AppCard v-if="generated !== null" class="settle flex flex-col items-center py-14 text-center">
        <div class="stamp grid size-16 place-items-center rounded-full bg-safe text-white shadow-lift">
          <CalendarCheck :size="28" :stroke-width="1.75" aria-hidden="true" />
        </div>
        <h2 class="display mt-5 text-3xl text-text">{{ plural(generated, 'fattura creata', 'fatture create') }}</h2>
        <p class="mt-1.5 text-base text-text-muted">Sono già emesse e aspettano il pagamento.</p>
        <div class="mt-7 flex gap-2">
          <AppButton variant="primary" :icon="FileText" :to="{ path: '/invoices', query: { status: 'issued', year } }">Vedi le fatture</AppButton>
          <AppButton @click="shift(-1)">Un altro mese</AppButton>
        </div>
      </AppCard>

      <div v-else class="grid grid-cols-1 items-start gap-5 lg:grid-cols-[1fr_19rem]">
        <AppCard :padded="false" class="settle overflow-hidden">
          <div class="flex items-center justify-between gap-4 px-5 pt-5 pb-4">
            <div>
              <h2 class="text-lg font-medium text-text">Sedute da fatturare</h2>
              <p class="text-sm text-text-subtle">Svolte a {{ label.toLocaleLowerCase('it-IT') }} e non ancora in fattura</p>
            </div>
            <label v-if="previews.length > 0" class="flex cursor-pointer items-center gap-2 text-sm text-text-muted">
              <input type="checkbox" :checked="allSelected" :indeterminate="!allSelected && selected.size > 0" @change="toggleAll" />
              Tutti
            </label>
          </div>

          <SkeletonRows v-if="loading" variant="list" :count="6" label="Caricamento delle sedute" />

          <EmptyState
            v-else-if="previews.length === 0"
            :icon="CalendarCheck"
            :bordered="false"
            :title="`Niente da fatturare per ${label.toLocaleLowerCase('it-IT')}`"
            description="Tutte le sedute svolte hanno già una fattura, oppure nessuna è segnata come svolta in agenda."
          >
            <AppButton to="/agenda">Apri l'agenda</AppButton>
          </EmptyState>

          <ul v-else class="border-t border-border">
            <li v-for="preview in previews" :key="preview.client_id" class="border-b border-border last:border-b-0">
              <label
                class="flex cursor-pointer items-start gap-3.5 px-5 py-3.5 transition-colors"
                :class="selected.has(preview.client_id) ? 'hover:bg-surface-hover' : 'bg-surface text-text-subtle'"
              >
                <input type="checkbox" class="mt-2" :checked="selected.has(preview.client_id)" @change="toggle(preview.client_id)" />
                <PatientMonogram :name="preview.client_name" size="md" />
                <span class="min-w-0 flex-1">
                  <span class="block text-base font-medium" :class="selected.has(preview.client_id) ? 'text-text' : 'text-text-muted'">{{ preview.client_name }}</span>
                  <span v-for="(line, index) in preview.lines" :key="index" class="block truncate text-sm text-text-subtle">
                    {{ line.quantity }} × {{ line.description }} · {{ formatCurrency(line.unit_price) }}
                  </span>
                </span>
                <span class="text-right">
                  <span class="tabular block text-base font-medium" :class="selected.has(preview.client_id) ? 'text-text' : 'text-text-subtle line-through'">{{ formatCurrency(preview.estimated_due) }}</span>
                  <span class="block text-xs text-text-subtle">{{ plural(preview.appointment_count, 'seduta', 'sedute') }}</span>
                </span>
              </label>
            </li>
          </ul>
        </AppCard>

        <aside class="settle lg:sticky lg:top-30" style="--settle: 1">
          <AppCard class="space-y-5">
            <FormField v-slot="{ id }" label="Metodo di pagamento">
              <select :id="id" v-model="paymentMethod" class="field">
                <option v-for="method in PAYMENT_METHOD_ORDER" :key="method" :value="method">{{ PAYMENT_METHOD_LABEL[method] }}</option>
              </select>
            </FormField>
            <ToggleSwitch v-model="applyEnpap" label="ENPAP 2%" description="Contributo integrativo su ogni fattura." />
            <div class="border-t border-border pt-4">
              <p class="label-quiet">{{ plural(chosen.length, 'fattura', 'fatture') }} · {{ plural(totals.sessions, 'seduta', 'sedute') }}</p>
              <p class="mt-0.5 text-[2rem] font-semibold leading-tight tracking-[-0.02em] text-text">{{ formatCurrency(totals.due) }}</p>
              <p class="text-xs text-text-subtle">Stima, calcolata con ENPAP e bollo</p>
            </div>
            <AppButton variant="primary" block :icon="Sparkles" :disabled="chosen.length === 0" @click="confirmOpen = true">
              Genera {{ plural(chosen.length, 'fattura', 'fatture') }}
            </AppButton>
          </AppCard>
        </aside>
      </div>
    </div>

    <ConfirmDialog
      :open="confirmOpen"
      title="Generare le fatture?"
      :blast-radius="`${plural(chosen.length, 'nuova fattura emessa', 'nuove fatture emesse')} per ${label.toLocaleLowerCase('it-IT')}, ${formatCurrency(totals.due)} in totale. Le sedute incluse non compariranno più tra quelle da fatturare.`"
      confirm-label="Genera"
      tone="accent"
      :loading="generating"
      @confirm="generate"
      @cancel="confirmOpen = false"
    />
  </div>
</template>

<style scoped>
.stamp { animation: stamp-land 560ms cubic-bezier(0.2, 1.4, 0.4, 1) both; }
@keyframes stamp-land {
  0% { transform: scale(1.5) rotate(-12deg); opacity: 0; }
  55% { transform: scale(0.92) rotate(2deg); opacity: 1; }
  100% { transform: none; }
}
</style>
