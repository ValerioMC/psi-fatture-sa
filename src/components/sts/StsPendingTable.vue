<script setup lang="ts">
/**
 * Paid invoices of the year the Sistema TS does not hold yet. Each row says
 * in advance what will go out: the amount, whether the patient opposed, and
 * whether a missing codice fiscale will stop it.
 */
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { CircleCheck, Send, ShieldOff, TriangleAlert, X } from 'lucide-vue-next'
import type { Client, Invoice } from '@/types'
import { useStsStore } from '@/stores/sts'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import TsMark from '@/components/ui/TsMark.vue'
import { formatCurrency, formatDate } from '@/utils/format'
import { plural } from '@/utils/labels'

const props = defineProps<{
  invoices: readonly Invoice[]
  clients: ReadonlyMap<number, Client>
  sending: boolean
}>()

const emit = defineEmits<{ send: [invoiceIds: number[]] }>()

const router = useRouter()
const sts = useStsStore()
const selected = ref(new Set<number>())

watch(() => props.invoices, () => { selected.value = new Set() })

interface Readiness {
  opposed: boolean
  blocker: string | null
}

function readiness(invoice: Invoice): Readiness {
  const client = props.clients.get(invoice.client_id)
  if (!client) return { opposed: false, blocker: null }
  if (client.client_type !== 'persona_fisica') return { opposed: false, blocker: 'Intestata a un’azienda' }
  if (!client.sts_authorization) return { opposed: true, blocker: null }
  return { opposed: false, blocker: client.fiscal_code.trim().length === 16 ? null : 'Manca il codice fiscale' }
}

const rows = computed(() => props.invoices.map((invoice) => ({ invoice, readiness: readiness(invoice), mark: sts.stateOf(invoice.id).kind })))
const sendable = computed(() => rows.value.filter((row) => row.readiness.blocker === null).map((row) => row.invoice.id))
const allSelected = computed(() => sendable.value.length > 0 && sendable.value.every((id) => selected.value.has(id)))
const someSelected = computed(() => selected.value.size > 0 && !allSelected.value)
const selectedTotal = computed(() =>
  props.invoices.filter((invoice) => selected.value.has(invoice.id)).reduce((sum, invoice) => sum + invoice.total_gross, 0),
)

function toggle(id: number): void {
  const next = new Set(selected.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selected.value = next
}

function toggleAll(): void {
  selected.value = allSelected.value ? new Set() : new Set(sendable.value)
}

function sendSelected(): void {
  emit('send', [...selected.value])
}
</script>

<template>
  <EmptyState
    v-if="rows.length === 0"
    :icon="CircleCheck"
    :bordered="false"
    title="Niente da trasmettere"
    description="Ogni fattura pagata di quest’anno è già sul Sistema TS o in coda."
  />

  <table v-else class="data-table text-base">
    <thead>
      <tr>
        <th class="w-12 pl-5 font-normal">
          <input type="checkbox" :checked="allSelected" :indeterminate="someSelected" :disabled="sendable.length === 0" aria-label="Seleziona tutte le fatture trasmissibili" @change="toggleAll" />
        </th>
        <th class="w-9 font-normal"><span class="sr-only">Sistema TS</span></th>
        <th class="w-24 font-medium">Numero</th>
        <th class="font-medium">Paziente</th>
        <th class="w-28 font-medium">Pagata il</th>
        <th class="w-32 pr-6 text-right font-medium">Spesa sanitaria</th>
        <th class="w-56 pr-5 font-medium">Note</th>
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="{ invoice, readiness: ready, mark } in rows"
        :key="invoice.id"
        class="group h-row cursor-pointer"
        data-row
        :data-selected="selected.has(invoice.id)"
        @click="router.push(`/invoices/${invoice.id}`)"
      >
        <td class="pl-5" @click.stop>
          <input
            type="checkbox"
            :checked="selected.has(invoice.id)"
            :disabled="ready.blocker !== null"
            :aria-label="`Seleziona la fattura N. ${invoice.invoice_number}`"
            @change="toggle(invoice.id)"
          />
        </td>
        <td><TsMark :kind="mark" :size="18" /></td>
        <td class="tabular text-text-muted">N. {{ invoice.invoice_number }}</td>
        <td class="truncate text-text">{{ invoice.client_name }}</td>
        <td class="tabular text-sm text-text-muted">{{ invoice.paid_date ? formatDate(invoice.paid_date) : '—' }}</td>
        <td class="tabular pr-6 text-right font-medium text-text">{{ formatCurrency(invoice.total_gross) }}</td>
        <td class="pr-5">
          <span v-if="ready.blocker" class="inline-flex items-center gap-1.5 text-xs text-danger">
            <TriangleAlert :size="13" aria-hidden="true" />{{ ready.blocker }}
          </span>
          <AppBadge v-else-if="ready.opposed" tone="neutral">
            <ShieldOff :size="11" class="mr-1 inline" aria-hidden="true" />Opposizione: senza CF
          </AppBadge>
          <span v-else-if="mark === 'rejected'" class="text-xs text-danger">Scartata in precedenza</span>
          <span v-else-if="mark === 'cancelled'" class="text-xs text-text-subtle">Annullata in precedenza</span>
        </td>
      </tr>
    </tbody>
  </table>

  <Teleport to="body">
    <Transition name="rise">
      <div
        v-if="selected.size > 0"
        class="fixed bottom-6 left-[calc(50%+var(--spacing-sidebar)/2)] z-(--z-overlay) flex -translate-x-1/2 items-center gap-3 rounded-card border border-border bg-surface-raised py-2 pl-4 pr-2 shadow-modal"
        role="region"
        aria-label="Trasmissione delle fatture selezionate"
      >
        <span class="whitespace-nowrap text-base font-medium text-text">{{ plural(selected.size, 'fattura', 'fatture') }}</span>
        <span class="tabular whitespace-nowrap text-sm text-text-muted">{{ formatCurrency(selectedTotal) }}</span>
        <span class="h-5 w-px bg-border" aria-hidden="true" />
        <AppButton variant="primary" size="sm" :icon="Send" :loading="sending" @click="sendSelected">Trasmetti al Sistema TS</AppButton>
        <AppButton variant="ghost" size="sm" :icon="X" label="Annulla selezione" @click="selected = new Set()" />
      </div>
    </Transition>
  </Teleport>
</template>
