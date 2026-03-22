<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { Plus, Pencil, Trash2, X, Check, Briefcase, Tag, TrendingUp } from 'lucide-vue-next'
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
        <!-- Filter pill toggle -->
        <button
          type="button"
          class="flex items-center gap-2.5 border border-sage-200 text-sage-600 hover:bg-sage-50 px-3.5 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer"
          @click="toggleFilter"
        >
          <div
            class="w-8 h-4.5 rounded-full transition-colors duration-200 relative shrink-0"
            :class="activeOnly ? 'bg-sage-500' : 'bg-sage-200'"
            style="height: 18px; width: 32px;"
          >
            <div
              class="absolute top-[3px] w-3 h-3 rounded-full bg-white shadow-sm transition-transform duration-200"
              :class="activeOnly ? 'translate-x-[14px]' : 'translate-x-[3px]'"
            />
          </div>
          <span>{{ activeOnly ? 'Solo attive' : 'Tutte' }}</span>
        </button>

        <button
          type="button"
          class="relative overflow-hidden bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-all hover:shadow-lg hover:shadow-sage-200 cursor-pointer"
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
            <Tag class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Attive</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.active }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #b88e67, #8a5f42)">
            <TrendingUp class="w-4 h-4 text-white" />
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
        <button
          type="button"
          class="mt-4 flex items-center gap-1.5 text-sm font-medium text-sage-600 hover:text-sage-800 transition-colors cursor-pointer"
          @click="openCreate"
        >
          <Plus class="w-4 h-4" />
          Aggiungi prestazione
        </button>
      </div>

      <!-- Card grid -->
      <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="(service, idx) in servicesStore.services"
          :key="service.id"
          :style="{ '--i': idx }"
          class="service-card rounded-2xl overflow-hidden border border-sage-100/60 bg-white/70 backdrop-blur-sm flex flex-col transition-all duration-200 hover:shadow-xl hover:-translate-y-1 cursor-pointer group"
          @click="openEdit(service)"
        >
          <!-- Gradient header -->
          <div
            class="px-5 pt-5 pb-4 relative overflow-hidden"
            :style="{ background: cardAccent(service.id) }"
          >
            <!-- Decorative circles -->
            <div class="absolute -right-5 -top-5 w-24 h-24 rounded-full bg-white/10 pointer-events-none" />
            <div class="absolute right-4 bottom-0 w-14 h-14 rounded-full bg-white/5 pointer-events-none" />

            <div class="relative flex items-start justify-between gap-2">
              <!-- Initial avatar -->
              <div class="w-10 h-10 rounded-xl bg-white/20 flex items-center justify-center text-white text-sm font-bold shrink-0 backdrop-blur-sm">
                {{ service.name.charAt(0).toUpperCase() }}
              </div>
              <!-- Status badge -->
              <span
                class="text-[10px] font-semibold px-2 py-0.5 rounded-full shrink-0 mt-0.5"
                :class="service.is_active
                  ? 'bg-white/25 text-white'
                  : 'bg-black/20 text-white/60'"
              >
                {{ service.is_active ? 'Attiva' : 'Inattiva' }}
              </span>
            </div>

            <div class="relative mt-3">
              <h3 class="text-sm font-semibold text-white leading-snug">{{ service.name }}</h3>
              <p class="text-[10px] text-white/65 mt-0.5">
                IVA {{ service.vat_rate > 0 ? service.vat_rate + '%' : 'esente' }}
              </p>
            </div>
          </div>

          <!-- Card body -->
          <div class="px-5 pt-4 pb-3 flex flex-col flex-1">
            <p class="text-xs text-sage-500 leading-relaxed flex-1 line-clamp-2 min-h-[2.25rem]">
              {{ service.description || 'Nessuna descrizione.' }}
            </p>

            <!-- Price + actions footer -->
            <div class="flex items-end justify-between pt-4 mt-3 border-t border-sage-100/50">
              <div>
                <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Tariffa</p>
                <p class="text-xl font-bold text-sage-900 tracking-tight leading-none tabular-nums">
                  {{ formatPrice(service.default_price) }}
                </p>
              </div>

              <!-- Actions: always visible -->
              <div class="flex gap-1" @click.stop>
                <button
                  type="button"
                  class="p-2 text-sage-300 hover:text-ocean-600 hover:bg-ocean-50 rounded-lg transition-all duration-150 cursor-pointer"
                  title="Modifica"
                  @click="openEdit(service)"
                >
                  <Pencil class="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  class="p-2 text-sage-300 hover:text-red-500 hover:bg-red-50 rounded-lg transition-all duration-150 cursor-pointer"
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

      <!-- Modal -->
      <Teleport to="body">
        <Transition name="modal">
          <div v-if="showModal" class="fixed inset-0 z-50 flex items-center justify-center">
            <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" @click="closeModal" />
            <div class="relative glass-card rounded-2xl shadow-2xl w-full max-w-lg mx-4 p-6 modal-panel">
              <!-- Modal header -->
              <div class="flex items-center justify-between mb-5">
                <div class="flex items-center gap-3">
                  <div
                    class="w-8 h-8 rounded-xl flex items-center justify-center shrink-0"
                    style="background: linear-gradient(135deg, #5d8062, #48654c)"
                  >
                    <Briefcase class="w-4 h-4 text-white" />
                  </div>
                  <h3 class="text-base font-semibold text-sage-900">
                    {{ editingService ? 'Modifica Prestazione' : 'Nuova Prestazione' }}
                  </h3>
                </div>
                <button
                  type="button"
                  class="p-1.5 text-sage-400 hover:text-sage-600 hover:bg-sage-50 rounded-lg transition-colors cursor-pointer"
                  @click="closeModal"
                >
                  <X class="w-4 h-4" />
                </button>
              </div>

              <form class="space-y-4" @submit.prevent="saveService">
                <!-- Name -->
                <div>
                  <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Nome *</label>
                  <input
                    v-model="form.name"
                    type="text"
                    required
                    placeholder="es. Visita di controllo"
                    class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                  />
                </div>

                <!-- Description -->
                <div>
                  <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Descrizione</label>
                  <input
                    v-model="form.description"
                    type="text"
                    placeholder="Descrizione opzionale"
                    class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                  />
                </div>

                <!-- Price + VAT -->
                <div class="grid grid-cols-2 gap-3">
                  <div>
                    <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Prezzo (€)</label>
                    <div class="relative">
                      <span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-sm text-sage-400 font-medium pointer-events-none">€</span>
                      <input
                        v-model.number="form.default_price"
                        type="number"
                        min="0"
                        step="0.01"
                        required
                        class="w-full border border-sage-200 rounded-xl pl-7 pr-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                      />
                    </div>
                  </div>
                  <div>
                    <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Aliquota IVA (%)</label>
                    <div class="relative">
                      <input
                        v-model.number="form.vat_rate"
                        type="number"
                        min="0"
                        max="100"
                        step="1"
                        required
                        class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                      />
                      <span class="absolute right-3.5 top-1/2 -translate-y-1/2 text-sm text-sage-400 font-medium pointer-events-none">%</span>
                    </div>
                  </div>
                </div>

                <!-- Active toggle -->
                <label class="flex items-center gap-3 cursor-pointer group/toggle p-3 rounded-xl hover:bg-sage-50/60 transition-colors">
                  <div
                    class="w-9 h-5 rounded-full transition-colors duration-200 relative shrink-0"
                    :class="form.is_active ? 'bg-sage-500' : 'bg-sage-200'"
                  >
                    <div
                      class="absolute top-[3px] w-3.5 h-3.5 rounded-full bg-white shadow-sm transition-transform duration-200"
                      :class="form.is_active ? 'translate-x-[17px]' : 'translate-x-[3px]'"
                    />
                  </div>
                  <input v-model="form.is_active" type="checkbox" class="sr-only" />
                  <div>
                    <p class="text-sm font-medium text-sage-800">Prestazione attiva</p>
                    <p class="text-xs text-sage-400">Visibile in fatture e appuntamenti</p>
                  </div>
                </label>

                <!-- Error -->
                <div v-if="formError" class="rounded-xl bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700 flex items-center gap-2">
                  <X class="w-4 h-4 shrink-0 text-red-400" />
                  {{ formError }}
                </div>

                <!-- Actions -->
                <div class="flex justify-end gap-3 pt-1">
                  <button
                    type="button"
                    class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors cursor-pointer"
                    @click="closeModal"
                  >
                    Annulla
                  </button>
                  <button
                    type="submit"
                    :disabled="saving"
                    class="bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-5 py-2 rounded-lg text-sm font-medium transition-all disabled:opacity-60 flex items-center gap-2 hover:shadow-md cursor-pointer"
                  >
                    <Check class="w-4 h-4" />
                    {{ saving ? 'Salvataggio...' : 'Salva' }}
                  </button>
                </div>
              </form>
            </div>
          </div>
        </Transition>
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

<style scoped>
/* Service cards: staggered entrance */
.service-card {
  animation: card-in 0.35s ease both;
  animation-delay: calc(var(--i, 0) * 55ms);
}

@keyframes card-in {
  from { opacity: 0; transform: translateY(14px); }
  to   { opacity: 1; transform: translateY(0);    }
}

/* Modal backdrop + panel */
.modal-enter-active { transition: opacity 0.2s ease; }
.modal-leave-active { transition: opacity 0.15s ease; }
.modal-enter-from,
.modal-leave-to     { opacity: 0; }

.modal-enter-active .modal-panel { transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.2s ease; }
.modal-leave-active .modal-panel { transition: transform 0.15s ease, opacity 0.15s ease; }
.modal-enter-from .modal-panel   { transform: translateY(16px) scale(0.97); opacity: 0; }
.modal-leave-to .modal-panel     { transform: translateY(8px)  scale(0.98); opacity: 0; }

@media (prefers-reduced-motion: reduce) {
  .service-card { animation: none; }
  .modal-enter-active,
  .modal-leave-active { transition: opacity 0.1s ease; }
  .modal-enter-active .modal-panel,
  .modal-leave-active .modal-panel { transition: opacity 0.1s ease; }
  .modal-enter-from .modal-panel,
  .modal-leave-to .modal-panel { transform: none; }
}
</style>
