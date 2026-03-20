<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { Plus, Pencil, Trash2, Briefcase, X, Check } from 'lucide-vue-next'
import { useServicesStore } from '@/stores/services'
import type { Service, CreateServiceInput, UpdateServiceInput } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import ConfirmModal from '@/components/ui/ConfirmModal.vue'

const servicesStore = useServicesStore()

const activeOnly = ref(true)
const serviceToDelete = ref<Service | null>(null)

// Modal state
const showModal = ref(false)
const editingService = ref<Service | null>(null)
const saving = ref(false)
const formError = ref<string | null>(null)

const emptyForm = (): CreateServiceInput => ({
  name: '',
  description: '',
  default_price: 0,
  vat_rate: 0,
  is_active: true,
})

const form = reactive<CreateServiceInput>(emptyForm())

function openCreate() {
  editingService.value = null
  Object.assign(form, emptyForm())
  formError.value = null
  showModal.value = true
}

function openEdit(service: Service) {
  editingService.value = service
  Object.assign(form, {
    name: service.name,
    description: service.description,
    default_price: service.default_price,
    vat_rate: service.vat_rate,
    is_active: service.is_active,
  })
  formError.value = null
  showModal.value = true
}

function closeModal() {
  showModal.value = false
  editingService.value = null
}

async function saveService() {
  saving.value = true
  formError.value = null
  try {
    if (editingService.value) {
      const input: UpdateServiceInput = { id: editingService.value.id, ...form }
      await servicesStore.editService(input)
    } else {
      await servicesStore.addService({ ...form })
    }
    closeModal()
    await servicesStore.fetchServices(activeOnly.value)
  } catch (e) {
    formError.value = String(e)
  } finally {
    saving.value = false
  }
}

async function toggleFilter() {
  activeOnly.value = !activeOnly.value
  await servicesStore.fetchServices(activeOnly.value)
}

function confirmDelete(service: Service) {
  serviceToDelete.value = service
}

async function handleDelete() {
  if (!serviceToDelete.value) return
  try {
    await servicesStore.removeService(serviceToDelete.value.id)
    await servicesStore.fetchServices(activeOnly.value)
  } finally {
    serviceToDelete.value = null
  }
}

function formatPrice(price: number): string {
  return new Intl.NumberFormat('it-IT', {
    style: 'currency',
    currency: 'EUR',
  }).format(price)
}

onMounted(() => servicesStore.fetchServices(activeOnly.value))
</script>

<template>
  <div class="p-8">
    <PageHeader title="Prestazioni" subtitle="Gestisci i servizi e le tariffe.">
      <button
        type="button"
        class="border border-gray-300 text-gray-700 hover:bg-gray-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        @click="toggleFilter"
      >
        {{ activeOnly ? 'Mostra tutte' : 'Solo attive' }}
      </button>
      <button
        type="button"
        class="bg-blue-600 text-white hover:bg-blue-700 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
        @click="openCreate"
      >
        <Plus class="w-4 h-4" />
        Nuova Prestazione
      </button>
    </PageHeader>

    <!-- Table -->
    <div class="bg-white rounded-xl border border-gray-100 shadow-sm">
      <div v-if="servicesStore.loading" class="px-6 py-12 text-center text-sm text-gray-400">
        Caricamento...
      </div>

      <div
        v-else-if="servicesStore.services.length === 0"
        class="flex flex-col items-center justify-center py-16 text-center"
      >
        <Briefcase class="w-12 h-12 text-gray-200 mb-3" />
        <p class="text-sm font-medium text-gray-500">Nessuna prestazione trovata</p>
        <p class="text-xs text-gray-400 mt-1">
          {{ activeOnly ? 'Non ci sono prestazioni attive.' : 'Aggiungi la prima prestazione.' }}
        </p>
      </div>

      <table v-else class="w-full text-sm">
        <thead>
          <tr class="border-b border-gray-100">
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Nome</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Descrizione</th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Prezzo</th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">IVA %</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Stato</th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Azioni</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-50">
          <tr v-for="service in servicesStore.services" :key="service.id" class="hover:bg-gray-50">
            <td class="px-6 py-3 font-medium text-gray-900">{{ service.name }}</td>
            <td class="px-6 py-3 text-gray-500 max-w-xs truncate">{{ service.description || '—' }}</td>
            <td class="px-6 py-3 text-right font-medium text-gray-900">{{ formatPrice(service.default_price) }}</td>
            <td class="px-6 py-3 text-right text-gray-500">{{ service.vat_rate }}%</td>
            <td class="px-6 py-3">
              <span
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
                :class="service.is_active ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-500'"
              >
                {{ service.is_active ? 'Attiva' : 'Inattiva' }}
              </span>
            </td>
            <td class="px-6 py-3">
              <div class="flex items-center justify-end gap-2">
                <button
                  type="button"
                  class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                  title="Modifica"
                  @click="openEdit(service)"
                >
                  <Pencil class="w-4 h-4" />
                </button>
                <button
                  type="button"
                  class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                  title="Elimina"
                  @click="confirmDelete(service)"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Service form modal -->
    <Teleport to="body">
      <div v-if="showModal" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/40" @click="closeModal" />
        <div class="relative bg-white rounded-xl shadow-xl w-full max-w-lg mx-4 p-6">
          <div class="flex items-center justify-between mb-5">
            <h3 class="text-base font-semibold text-gray-900">
              {{ editingService ? 'Modifica Prestazione' : 'Nuova Prestazione' }}
            </h3>
            <button type="button" class="text-gray-400 hover:text-gray-600" @click="closeModal">
              <X class="w-5 h-5" />
            </button>
          </div>

          <form class="space-y-4" @submit.prevent="saveService">
            <div>
              <label class="text-sm font-medium text-gray-700 block mb-1">Nome *</label>
              <input
                v-model="form.name"
                type="text"
                required
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-gray-700 block mb-1">Descrizione</label>
              <input
                v-model="form.description"
                type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="text-sm font-medium text-gray-700 block mb-1">Prezzo default (€)</label>
                <input
                  v-model.number="form.default_price"
                  type="number"
                  min="0"
                  step="0.01"
                  required
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label class="text-sm font-medium text-gray-700 block mb-1">Aliquota IVA (%)</label>
                <input
                  v-model.number="form.vat_rate"
                  type="number"
                  min="0"
                  max="100"
                  step="1"
                  required
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="form.is_active"
                type="checkbox"
                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span class="text-sm text-gray-700">Prestazione attiva</span>
            </label>

            <div v-if="formError" class="rounded-lg bg-red-50 border border-red-200 px-3 py-2 text-sm text-red-700">
              {{ formError }}
            </div>

            <div class="flex justify-end gap-3 pt-2">
              <button
                type="button"
                class="border border-gray-300 text-gray-700 hover:bg-gray-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                @click="closeModal"
              >
                Annulla
              </button>
              <button
                type="submit"
                :disabled="saving"
                class="bg-blue-600 text-white hover:bg-blue-700 px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-60 flex items-center gap-2"
              >
                <Check class="w-4 h-4" />
                {{ saving ? 'Salvataggio...' : 'Salva' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <ConfirmModal
      :open="!!serviceToDelete"
      title="Elimina prestazione"
      :message="`Sei sicuro di voler eliminare la prestazione '${serviceToDelete?.name}'?`"
      @confirm="handleDelete"
      @cancel="serviceToDelete = null"
    />
  </div>
</template>
