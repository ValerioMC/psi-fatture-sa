<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { Plus, Pencil, Trash2, X, Check, Briefcase, Tag, TrendingUp } from 'lucide-vue-next'
import { useServicesStore } from '@/stores/services'
import type { Service, CreateServiceInput, UpdateServiceInput } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import ConfirmModal from '@/components/ui/ConfirmModal.vue'
import { formatCurrency } from '@/utils/format'

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

const AVATAR_GRADIENTS = [
  'linear-gradient(135deg, #059669, #047857)',
  'linear-gradient(135deg, #4f46e5, #4338ca)',
  'linear-gradient(135deg, #78716c, #57534e)',
  'linear-gradient(135deg, #10b981, #059669)',
  'linear-gradient(135deg, #d97706, #b45309)',
  'linear-gradient(135deg, #6366f1, #4f46e5)',
]

function avatarGradient(id: number): string {
  return AVATAR_GRADIENTS[id % AVATAR_GRADIENTS.length]
}

function rowDelay(idx: number): number {
  return Math.min(idx, 10)
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

async function setFilter(value: boolean) {
  activeOnly.value = value
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

onMounted(() => servicesStore.fetchServices(activeOnly.value))
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">
      <PageHeader title="Prestazioni" subtitle="Catalogo dei servizi e tariffe.">
        <button
          type="button"
          class="group relative overflow-hidden text-white font-semibold px-4 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 cursor-pointer focus:outline-none"
          style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
          @click="openCreate"
        >
          <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
          <Plus class="w-4 h-4 relative z-10" />
          <span class="relative z-10">Nuova Prestazione</span>
        </button>
      </PageHeader>

      <!-- Stats strip -->
      <div class="grid grid-cols-3 gap-4 mb-5 animate-in">
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #4f46e5, #4338ca)">
            <Briefcase class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Prestazioni</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.total }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #059669, #047857)">
            <Tag class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Attive</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.active }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #d97706, #b45309)">
            <TrendingUp class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Tariffa media</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ formatCurrency(stats.avgPrice) }}</p>
          </div>
        </div>
      </div>

      <!-- Filter + count row -->
      <div class="flex items-center gap-3 mb-4 animate-in-d1">
        <div class="flex rounded-xl border border-sage-200 p-0.5 bg-sage-50/50 gap-0.5">
          <button
            type="button"
            class="px-3 py-1.5 text-xs font-semibold rounded-lg transition-all duration-200 cursor-pointer"
            :class="activeOnly ? 'bg-white shadow-sm text-sage-900' : 'text-sage-500 hover:text-sage-700'"
            @click="setFilter(true)"
          >
            Solo attive
          </button>
          <button
            type="button"
            class="px-3 py-1.5 text-xs font-semibold rounded-lg transition-all duration-200 cursor-pointer"
            :class="!activeOnly ? 'bg-white shadow-sm text-sage-900' : 'text-sage-500 hover:text-sage-700'"
            @click="setFilter(false)"
          >
            Tutte
          </button>
        </div>
        <span class="text-xs text-sage-400 whitespace-nowrap shrink-0 ml-auto">
          {{ servicesStore.services.length }} {{ servicesStore.services.length === 1 ? 'prestazione' : 'prestazioni' }}
        </span>
      </div>

      <!-- Table card -->
      <div class="glass-card rounded-2xl shadow-sm overflow-hidden animate-in-d1">

        <!-- Loading -->
        <div v-if="servicesStore.loading" class="flex flex-col items-center justify-center py-16 gap-3">
          <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
          <p class="text-sm text-sage-400">Caricamento prestazioni…</p>
        </div>

        <!-- Empty state -->
        <div
          v-else-if="servicesStore.services.length === 0"
          class="flex flex-col items-center justify-center py-16 text-center px-6"
        >
          <div class="w-14 h-14 rounded-2xl bg-sage-50 flex items-center justify-center mb-3">
            <Briefcase class="w-7 h-7 text-sage-300" />
          </div>
          <p class="text-sm font-semibold text-sage-600">Nessuna prestazione trovata</p>
          <p class="text-xs text-sage-400 mt-1">
            {{ activeOnly ? 'Non ci sono prestazioni attive.' : 'Aggiungi la prima prestazione.' }}
          </p>
          <button
            v-if="!activeOnly"
            type="button"
            class="mt-4 flex items-center gap-1.5 text-sm font-medium text-sage-600 hover:text-sage-800 transition-colors cursor-pointer"
            @click="openCreate"
          >
            <Plus class="w-4 h-4" />
            Aggiungi prestazione
          </button>
          <button
            v-else
            type="button"
            class="mt-3 text-xs text-sage-400 hover:text-sage-600 transition-colors cursor-pointer underline underline-offset-2"
            @click="setFilter(false)"
          >
            Mostra tutte le prestazioni
          </button>
        </div>

        <!-- Table -->
        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-sage-100/60 bg-sage-50/30">
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Prestazione</th>
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Descrizione</th>
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">IVA</th>
              <th class="px-5 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Tariffa</th>
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Stato</th>
              <th class="px-5 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Azioni</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-sage-50">
            <tr
              v-for="(service, idx) in servicesStore.services"
              :key="service.id"
              :style="{ '--i': rowDelay(idx) }"
              class="service-row hover:bg-sage-50/60 transition-all duration-150 cursor-pointer group"
              @click="openEdit(service)"
            >
              <!-- Avatar + Name -->
              <td class="px-5 py-3.5">
                <div class="flex items-center gap-3">
                  <div
                    class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0 text-white text-xs font-bold shadow-sm"
                    :style="{ background: avatarGradient(service.id) }"
                  >
                    {{ service.name.charAt(0).toUpperCase() }}
                  </div>
                  <p class="font-semibold text-sage-900 leading-tight">{{ service.name }}</p>
                </div>
              </td>

              <!-- Description -->
              <td class="px-5 py-3.5 max-w-[220px]">
                <p class="text-xs text-sage-500 truncate">{{ service.description || '—' }}</p>
              </td>

              <!-- VAT -->
              <td class="px-5 py-3.5">
                <span
                  class="inline-flex items-center text-[11px] font-semibold px-2 py-0.5 rounded-lg"
                  :class="service.vat_rate > 0
                    ? 'bg-ocean-50 text-ocean-600 border border-ocean-100'
                    : 'bg-sage-50 text-sage-400 border border-sage-100'"
                >
                  {{ service.vat_rate > 0 ? service.vat_rate + '%' : 'Esente' }}
                </span>
              </td>

              <!-- Price -->
              <td class="px-5 py-3.5 text-right">
                <span class="text-base font-bold text-sage-900 tabular-nums tracking-tight">
                  {{ formatCurrency(service.default_price) }}
                </span>
              </td>

              <!-- Status -->
              <td class="px-5 py-3.5">
                <span
                  class="inline-flex items-center gap-1 text-[11px] font-semibold px-2.5 py-0.5 rounded-full"
                  :class="service.is_active
                    ? 'bg-emerald-50 text-emerald-700 border border-emerald-100'
                    : 'bg-warm-100 text-warm-600 border border-warm-200'"
                >
                  <span
                    class="w-1.5 h-1.5 rounded-full shrink-0"
                    :class="service.is_active ? 'bg-emerald-500' : 'bg-warm-400'"
                  />
                  {{ service.is_active ? 'Attiva' : 'Inattiva' }}
                </span>
              </td>

              <!-- Actions -->
              <td class="px-5 py-3.5" @click.stop>
                <div class="flex items-center justify-end gap-1">
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
              </td>
            </tr>
          </tbody>
        </table>
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
                  <div class="w-8 h-8 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #4f46e5, #4338ca)">
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
                    placeholder="es. Seduta di psicoterapia"
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
                    <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Tariffa (€)</label>
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
                        class="w-full border border-sage-200 rounded-xl px-3.5 pr-8 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                      />
                      <span class="absolute right-3.5 top-1/2 -translate-y-1/2 text-sm text-sage-400 font-medium pointer-events-none">%</span>
                    </div>
                  </div>
                </div>

                <!-- Active toggle -->
                <label class="flex items-center gap-3 cursor-pointer p-3.5 rounded-xl border border-sage-100/60 hover:bg-sage-50/60 transition-colors">
                  <div
                    class="w-9 h-5 rounded-full transition-colors duration-200 relative shrink-0"
                    :class="form.is_active ? 'bg-emerald-500' : 'bg-sage-200'"
                  >
                    <div
                      class="absolute top-[3px] w-3.5 h-3.5 rounded-full bg-white shadow-sm transition-transform duration-200"
                      :class="form.is_active ? 'translate-x-[17px]' : 'translate-x-[3px]'"
                    />
                  </div>
                  <input v-model="form.is_active" type="checkbox" class="sr-only" />
                  <div class="flex-1">
                    <p class="text-sm font-medium text-sage-800">Prestazione attiva</p>
                    <p class="text-xs text-sage-400">Visibile in fatture e appuntamenti</p>
                  </div>
                  <span
                    v-if="form.is_active"
                    class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-700 border border-emerald-100 shrink-0"
                  >
                    Attiva
                  </span>
                </label>

                <!-- Error -->
                <div v-if="formError" class="rounded-xl bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700 flex items-center gap-2">
                  <X class="w-4 h-4 shrink-0 text-red-400" />
                  {{ formError }}
                </div>

                <!-- Actions -->
                <div class="flex justify-between items-center pt-1">
                  <button
                    type="button"
                    class="flex items-center gap-1.5 text-sm text-sage-500 hover:text-sage-700 transition-colors cursor-pointer"
                    @click="closeModal"
                  >
                    Annulla
                  </button>
                  <button
                    type="submit"
                    :disabled="saving"
                    class="group relative overflow-hidden text-white font-semibold px-5 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 disabled:opacity-60 cursor-pointer focus:outline-none"
                    style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
                  >
                    <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
                    <Check class="w-4 h-4 relative z-10" />
                    <span class="relative z-10">{{ saving ? 'Salvataggio…' : 'Salva' }}</span>
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
.service-row {
  animation: row-in 0.3s ease both;
  animation-delay: calc(var(--i, 0) * 35ms);
}

@keyframes row-in {
  from { opacity: 0; transform: translateX(-6px); }
  to   { opacity: 1; transform: translateX(0); }
}

.modal-enter-active { transition: opacity 0.2s ease; }
.modal-leave-active { transition: opacity 0.15s ease; }
.modal-enter-from,
.modal-leave-to     { opacity: 0; }

.modal-enter-active .modal-panel { transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.2s ease; }
.modal-leave-active .modal-panel { transition: transform 0.15s ease, opacity 0.15s ease; }
.modal-enter-from .modal-panel   { transform: translateY(16px) scale(0.97); opacity: 0; }
.modal-leave-to .modal-panel     { transform: translateY(8px)  scale(0.98); opacity: 0; }

@media (prefers-reduced-motion: reduce) {
  .service-row { animation: none; }
  .modal-enter-active,
  .modal-leave-active { transition: opacity 0.1s ease; }
  .modal-enter-active .modal-panel,
  .modal-leave-active .modal-panel { transition: opacity 0.1s ease; }
  .modal-enter-from .modal-panel,
  .modal-leave-to .modal-panel { transform: none; }
}
</style>
