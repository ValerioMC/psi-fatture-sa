<script setup lang="ts">
/**
 * The price list. A service is switched on or off right in its row: an
 * inactive service stays on old invoices but is no longer offered in new ones.
 */
import { computed, onMounted, reactive, ref } from 'vue'
import { ClipboardList, Pencil, Plus, Trash2 } from 'lucide-vue-next'
import { useServicesStore } from '@/stores/services'
import { useToastStore } from '@/stores/toast'
import type { CreateServiceInput, Service } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import FormField from '@/components/ui/FormField.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import type { SegmentOption } from '@/components/ui/types'
import { formatCurrency } from '@/utils/format'
import { plural } from '@/utils/labels'

const servicesStore = useServicesStore()
const toast = useToastStore()

type Filter = 'active' | 'all'
const filter = ref<Filter>('active')

onMounted(async () => {
  await servicesStore.fetchServices(false)
  if (servicesStore.error) toast.notifyError(servicesStore.error, 'Caricamento non riuscito')
})

const activeCount = computed(() => servicesStore.services.filter((service) => service.is_active).length)
const FILTERS = computed<SegmentOption<Filter>[]>(() => [
  { value: 'active', label: 'Attive', count: activeCount.value },
  { value: 'all', label: 'Tutte', count: servicesStore.services.length },
])

const visible = computed(() =>
  [...servicesStore.services]
    .filter((service) => filter.value === 'all' || service.is_active)
    .sort((a, b) => Number(b.is_active) - Number(a.is_active) || a.name.localeCompare(b.name, 'it')),
)

// ─── Inline activation ──────────────────────────────────────────────────────

const toggling = ref<Set<number>>(new Set())

async function setActive(service: Service, isActive: boolean): Promise<void> {
  toggling.value = new Set(toggling.value).add(service.id)
  try {
    await servicesStore.editService({ ...service, is_active: isActive })
    toast.notify(isActive ? `“${service.name}” di nuovo disponibile` : `“${service.name}” archiviata`)
  } catch (error) {
    toast.notifyError(error, 'Aggiornamento non riuscito')
  } finally {
    const next = new Set(toggling.value)
    next.delete(service.id)
    toggling.value = next
  }
}

// ─── Create and edit ────────────────────────────────────────────────────────

const dialogOpen = ref(false)
const editing = ref<Service | null>(null)
const saving = ref(false)
const nameError = ref<string | undefined>(undefined)
const form = reactive<CreateServiceInput>({ name: '', description: '', default_price: 0, vat_rate: 0, is_active: true })

function openCreate(): void {
  editing.value = null
  Object.assign(form, { name: '', description: '', default_price: 0, vat_rate: 0, is_active: true })
  nameError.value = undefined
  dialogOpen.value = true
}

function openEdit(service: Service): void {
  editing.value = service
  Object.assign(form, {
    name: service.name,
    description: service.description,
    default_price: service.default_price,
    vat_rate: service.vat_rate,
    is_active: service.is_active,
  })
  nameError.value = undefined
  dialogOpen.value = true
}

async function save(): Promise<void> {
  if (form.name.trim() === '') {
    nameError.value = 'Dai un nome alla prestazione.'
    return
  }
  saving.value = true
  try {
    if (editing.value) await servicesStore.editService({ id: editing.value.id, ...form })
    else await servicesStore.addService({ ...form })
    toast.notify(editing.value ? 'Prestazione aggiornata' : 'Prestazione aggiunta')
    dialogOpen.value = false
  } catch (error) {
    toast.notifyError(error, 'Salvataggio non riuscito')
  } finally {
    saving.value = false
  }
}

// ─── Delete ─────────────────────────────────────────────────────────────────

const toDelete = ref<Service | null>(null)
const deleting = ref(false)

async function confirmDelete(): Promise<void> {
  const service = toDelete.value
  if (service === null) return
  deleting.value = true
  try {
    await servicesStore.removeService(service.id)
    toast.notify(`“${service.name}” eliminata`)
    toDelete.value = null
  } catch (error) {
    toast.notifyError(error, 'Eliminazione non riuscita')
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div>
    <PageHeader title="Prestazioni" subtitle="Il tuo listino: nome, tariffa e IVA proposti quando scrivi una fattura.">
      <AppButton variant="primary" :icon="Plus" @click="openCreate">Nuova prestazione</AppButton>
    </PageHeader>

    <div class="mx-auto max-w-[72rem] px-8 pt-6 pb-12">
      <div class="settle mb-3">
        <SegmentedControl v-model="filter" :options="FILTERS" label="Mostra" size="sm" />
      </div>

      <AppCard :padded="false" class="settle overflow-hidden" style="--settle: 1">
        <SkeletonRows v-if="servicesStore.loading" variant="list" :count="6" label="Caricamento delle prestazioni" />

        <EmptyState
          v-else-if="visible.length === 0"
          :icon="ClipboardList"
          :bordered="false"
          :title="servicesStore.services.length === 0 ? 'Il listino è vuoto' : 'Nessuna prestazione attiva'"
          description="Aggiungi le prestazioni che offri: le ritroverai già compilate in fattura e in agenda."
        >
          <AppButton variant="primary" :icon="Plus" @click="openCreate">Nuova prestazione</AppButton>
        </EmptyState>

        <table v-else class="w-full text-base">
          <thead class="border-b border-border bg-surface text-left text-xs text-text-subtle">
            <tr class="h-9">
              <th class="pl-5 font-medium">Prestazione</th>
              <th class="w-24 font-medium">IVA</th>
              <th class="w-32 pr-8 text-right font-medium">Tariffa</th>
              <th class="w-24 font-medium">Attiva</th>
              <th class="w-24 pr-4"><span class="sr-only">Azioni</span></th>
            </tr>
          </thead>
          <TransitionGroup tag="tbody" name="list">
            <tr
              v-for="service in visible"
              :key="service.id"
              class="group h-14 cursor-pointer border-b border-border last:border-b-0 transition-colors hover:bg-surface-hover"
              @click="openEdit(service)"
            >
              <td class="pl-5">
                <span class="block font-medium" :class="service.is_active ? 'text-text' : 'text-text-subtle'">{{ service.name }}</span>
                <span v-if="service.description" class="block truncate text-xs text-text-subtle">{{ service.description }}</span>
              </td>
              <td class="text-sm text-text-muted">{{ service.vat_rate > 0 ? `${service.vat_rate}%` : 'Esente' }}</td>
              <td class="tabular pr-8 text-right font-medium" :class="service.is_active ? 'text-text' : 'text-text-subtle'">{{ formatCurrency(service.default_price) }}</td>
              <td @click.stop>
                <ToggleSwitch
                  :model-value="service.is_active"
                  compact
                  :label="`${service.name} attiva`"
                  :disabled="toggling.has(service.id)"
                  @update:model-value="setActive(service, $event)"
                />
              </td>
              <td class="pr-4" @click.stop>
                <div class="flex justify-end gap-0.5 opacity-60 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
                  <AppButton variant="ghost" size="sm" :icon="Pencil" :label="`Modifica ${service.name}`" @click="openEdit(service)" />
                  <AppButton variant="danger-quiet" size="sm" :icon="Trash2" :label="`Elimina ${service.name}`" @click="toDelete = service" />
                </div>
              </td>
            </tr>
          </TransitionGroup>
        </table>
      </AppCard>
      <p v-if="!servicesStore.loading && servicesStore.services.length > 0" class="mt-3 px-1 text-xs text-text-subtle">
        {{ plural(servicesStore.services.length - activeCount, 'prestazione archiviata', 'prestazioni archiviate') }}: restano nelle fatture passate, non vengono più proposte.
      </p>
    </div>

    <AppDialog
      :open="dialogOpen"
      :title="editing ? 'Modifica prestazione' : 'Nuova prestazione'"
      @close="dialogOpen = false"
    >
      <form id="service-form" class="space-y-5" novalidate @submit.prevent="save">
        <FormField v-slot="{ id, invalid, describedBy }" label="Nome" required :error="nameError">
          <input :id="id" v-model="form.name" type="text" class="field" placeholder="Psicoterapia individuale" data-autofocus :aria-invalid="invalid" :aria-describedby="describedBy" />
        </FormField>
        <FormField v-slot="{ id }" label="Descrizione" optional>
          <input :id="id" v-model="form.description" type="text" class="field" placeholder="Seduta di 50 minuti" />
        </FormField>
        <div class="grid grid-cols-2 gap-4">
          <FormField v-slot="{ id }" label="Tariffa">
            <div class="relative">
              <input :id="id" v-model.number="form.default_price" type="number" min="0" step="0.01" class="field tabular pr-8 text-right" />
              <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-sm text-text-subtle">€</span>
            </div>
          </FormField>
          <FormField v-slot="{ id }" label="IVA" hint="0% per le prestazioni sanitarie esenti.">
            <div class="relative">
              <input :id="id" v-model.number="form.vat_rate" type="number" min="0" max="100" step="1" class="field tabular pr-8 text-right" />
              <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-sm text-text-subtle">%</span>
            </div>
          </FormField>
        </div>
        <ToggleSwitch v-model="form.is_active" label="Attiva" description="Proposta quando scrivi fatture e appuntamenti." />
      </form>
      <template #footer>
        <div class="flex-1" />
        <AppButton variant="ghost" @click="dialogOpen = false">Annulla</AppButton>
        <AppButton variant="primary" type="submit" form="service-form" :loading="saving">{{ editing ? 'Salva' : 'Aggiungi' }}</AppButton>
      </template>
    </AppDialog>

    <ConfirmDialog
      :open="toDelete !== null"
      title="Eliminare la prestazione?"
      :message="toDelete ? `Se vuoi solo smettere di proporla, spegnila: resta nello storico.` : undefined"
      :blast-radius="toDelete ? `“${toDelete.name}” sparirà dal listino. Le fatture che la contengono restano intatte, con la loro descrizione.` : undefined"
      :loading="deleting"
      @confirm="confirmDelete"
      @cancel="toDelete = null"
    />
  </div>
</template>
