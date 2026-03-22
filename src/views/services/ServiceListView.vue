<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { Plus, Pencil, Trash2, X, Check, Briefcase } from 'lucide-vue-next'
import { useServicesStore } from '@/stores/services'
import type { Service, CreateServiceInput, UpdateServiceInput } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import ConfirmModal from '@/components/ui/ConfirmModal.vue'

const servicesStore = useServicesStore()

const activeOnly = ref(true)
const serviceToDelete = ref<Service | null>(null)

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

const stats = computed(() => {
  const all = servicesStore.services
  const prices = all.filter(s => s.default_price > 0).map(s => s.default_price)
  return {
    total: all.length,
    active: all.filter(s => s.is_active).length,
    avgPrice: prices.length ? prices.reduce((a, b) => a + b, 0) / prices.length : 0,
  }
})

const CARD_ACCENTS = [
  'linear-gradient(135deg, #5d8062, #48654c)',
  'linear-gradient(135deg, #0c8aeb, #0153a2)',
  'linear-gradient(135deg, #b88e67, #8a5f42)',
  'linear-gradient(135deg, #d4a017, #a16207)',
  'linear-gradient(135deg, #7a9b7e, #3a513e)',
  'linear-gradient(135deg, #36a5fa, #0153a2)',
]

function cardAccent(id: number): string {
  return CARD_ACCENTS[id % CARD_ACCENTS.length]
}

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
  return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(price)
}

onMounted(() => servicesStore.fetchServices(activeOnly.value))
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">
      <PageHeader title="Prestazioni" subtitle="Catalogo dei servizi e tariffe.">
        <button
          type="button"
          class="border border-sage-200 text-sage-600 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
          @click="toggleFilter"
        >
          {{ activeOnly ? 'Mostra tutte' : 'Solo attive' }}
        </button>
        <button
          type="button"
          class="relative overflow-hidden bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-all hover:shadow-lg hover:shadow-sage-200"
          @click="openCreate"
        >
          <Plus class="w-4 h-4" />
          Nuova Prestazione
        </button>
      </PageHeader>

      <!-- Stats strip -->
      <div class="grid grid-cols-3 gap-4 mb-6 animate-in">
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #5d8062, #48654c)">
            <Briefcase class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Prestazioni</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.total }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #0c8aeb, #0153a2)">
            <Check class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Attive</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.active }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #b88e67, #8a5f42)">
            <span class="text-white text-xs font-bold">€</span>
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Tariffa media</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ formatPrice(stats.avgPrice) }}</p>
          </div>
        </div>
      </div>

      <!-- Loading -->
      <div v-if="servicesStore.loading" class="flex flex-col items-center justify-center py-20 gap-3">
        <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        <p class="text-sm text-sage-400">Caricamento...</p>
      </div>

      <!-- Empty state -->
      <div
        v-else-if="servicesStore.services.length === 0"
        class="flex flex-col items-center justify-center py-20 text-center animate-in"
      >
        <div class="w-14 h-14 rounded-2xl bg-sage-50 flex items-center justify-center mb-3">
          <Briefcase class="w-7 h-7 text-sage-300" />
        </div>
        <p class="text-sm font-semibold text-sage-600">Nessuna prestazione</p>
        <p class="text-xs text-sage-400 mt-1">
          {{ activeOnly ? 'Non ci sono prestazioni attive.' : 'Aggiungi la prima prestazione.' }}
        </p>
      </div>

      <!-- Card grid -->
      <div v-else class="grid grid-cols-3 gap-4 animate-in-d1">
        <div
          v-for="service in servicesStore.services"
          :key="service.id"
          class="glass-card rounded-2xl overflow-hidden shadow-sm hover-lift group flex flex-col"
        >
          <!-- Colored accent bar -->
          <div class="h-1" :style="{ background: cardAccent(service.id) }" />

          <div class="p-5 flex flex-col flex-1">
            <!-- Initial + Name + Price -->
            <div class="flex items-start gap-3 mb-3">
              <!-- Service initial circle -->
              <div
                class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0 text-white text-sm font-bold"
                :style="{ background: cardAccent(service.id) }"
              >
                {{ service.name.charAt(0).toUpperCase() }}
              </div>
              <div class="flex-1 min-w-0">
                <h3 class="text-sm font-semibold text-sage-900 leading-snug">{{ service.name }}</h3>
                <p class="text-[10px] text-sage-400 mt-0.5">
                  IVA {{ service.vat_rate > 0 ? service.vat_rate + '%' : 'esente' }}
                </p>
              </div>
            </div>

            <!-- Description -->
            <p class="text-xs text-sage-500 leading-relaxed flex-1 mb-4 line-clamp-2 min-h-[2.25rem]">
              {{ service.description || 'Nessuna descrizione.' }}
            </p>

            <!-- Price + Status + Actions -->
            <div class="flex items-end justify-between mt-auto pt-3 border-t border-sage-100/60">
              <div>
                <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Tariffa</p>
                <p class="text-2xl font-bold text-sage-900 tracking-tight leading-none">
                  {{ formatPrice(service.default_price) }}
                </p>
              </div>

              <div class="flex flex-col items-end gap-2">
                <span
                  class="text-[10px] font-semibold px-2 py-0.5 rounded-full"
                  :class="service.is_active
                    ? 'bg-sage-100 text-sage-700'
                    : 'bg-warm-100/80 text-warm-500'"
                >
                  {{ service.is_active ? 'Attiva' : 'Inattiva' }}
                </span>
                <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                  <button
                    type="button"
                    class="p-1.5 text-sage-400 hover:text-ocean-600 hover:bg-ocean-50 rounded-lg transition-colors"
                    title="Modifica"
                    @click="openEdit(service)"
                  >
                    <Pencil class="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    class="p-1.5 text-sage-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                    title="Elimina"
                    @click="confirmDelete(service)"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal -->
      <Teleport to="body">
        <div v-if="showModal" class="fixed inset-0 z-50 flex items-center justify-center">
          <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" @click="closeModal" />
          <div class="relative glass-card rounded-2xl shadow-2xl w-full max-w-lg mx-4 p-6 animate-in">
            <div class="flex items-center justify-between mb-5">
              <h3 class="text-base font-semibold text-sage-900">
                {{ editingService ? 'Modifica Prestazione' : 'Nuova Prestazione' }}
              </h3>
              <button type="button" class="text-sage-400 hover:text-sage-600 transition-colors" @click="closeModal">
                <X class="w-5 h-5" />
              </button>
            </div>

            <form class="space-y-4" @submit.prevent="saveService">
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Nome *</label>
                <input
                  v-model="form.name"
                  type="text"
                  required
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80"
                />
              </div>
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Descrizione</label>
                <input
                  v-model="form.description"
                  type="text"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80"
                />
              </div>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="text-sm font-medium text-sage-700 block mb-1">Prezzo default (€)</label>
                  <input
                    v-model.number="form.default_price"
                    type="number"
                    min="0"
                    step="0.01"
                    required
                    class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80"
                  />
                </div>
                <div>
                  <label class="text-sm font-medium text-sage-700 block mb-1">Aliquota IVA (%)</label>
                  <input
                    v-model.number="form.vat_rate"
                    type="number"
                    min="0"
                    max="100"
                    step="1"
                    required
                    class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80"
                  />
                </div>
              </div>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="form.is_active"
                  type="checkbox"
                  class="rounded border-sage-300 text-sage-600 focus:ring-sage-400/40"
                />
                <span class="text-sm text-sage-700">Prestazione attiva</span>
              </label>

              <div v-if="formError" class="rounded-lg bg-red-50 border border-red-200 px-3 py-2 text-sm text-red-700">
                {{ formError }}
              </div>

              <div class="flex justify-end gap-3 pt-2">
                <button
                  type="button"
                  class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                  @click="closeModal"
                >
                  Annulla
                </button>
                <button
                  type="submit"
                  :disabled="saving"
                  class="bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-2 rounded-lg text-sm font-medium transition-all disabled:opacity-60 flex items-center gap-2"
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
        :message="`Sei sicuro di voler eliminare '${serviceToDelete?.name}'?`"
        @confirm="handleDelete"
        @cancel="serviceToDelete = null"
      />
    </div>
  </div>
</template>
