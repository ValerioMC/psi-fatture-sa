<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Search, Plus, Pencil, Trash2, Users, Mail, ShieldCheck, MapPin, X } from 'lucide-vue-next'
import { useClientsStore } from '@/stores/clients'
import type { Client } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import ConfirmModal from '@/components/ui/ConfirmModal.vue'

const router = useRouter()
const clientsStore = useClientsStore()

const searchQuery = ref('')
const clientToDelete = ref<Client | null>(null)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

const stats = computed(() => {
  const all = clientsStore.clients
  return {
    total: all.length,
    withEmail: all.filter(c => c.email).length,
    stsAuthorized: all.filter(c => c.sts_authorization).length,
  }
})

onMounted(() => clientsStore.fetchClients())

function onSearchInput() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    clientsStore.fetchClients(searchQuery.value || undefined)
  }, 300)
}

function clearSearch() {
  searchQuery.value = ''
  clientsStore.fetchClients(undefined)
}

function confirmDelete(client: Client) {
  clientToDelete.value = client
}

async function handleDelete() {
  if (!clientToDelete.value) return
  try {
    await clientsStore.removeClient(clientToDelete.value.id)
  } finally {
    clientToDelete.value = null
  }
}

function displayName(client: Client): string {
  if (client.client_type === 'azienda') return client.last_name
  return `${client.last_name} ${client.first_name}`.trim()
}

function initials(client: Client): string {
  if (client.client_type === 'azienda') return client.last_name.slice(0, 2).toUpperCase()
  const l = client.last_name.charAt(0)
  const f = client.first_name.charAt(0)
  return `${l}${f}`.toUpperCase()
}

const AVATAR_GRADIENTS = [
  'linear-gradient(135deg, #059669, #047857)',
  'linear-gradient(135deg, #4f46e5, #4338ca)',
  'linear-gradient(135deg, #78716c, #57534e)',
  'linear-gradient(135deg, #10b981, #059669)',
  'linear-gradient(135deg, #6366f1, #4f46e5)',
  'linear-gradient(135deg, #d97706, #b45309)',
]

function avatarGradient(id: number): string {
  return AVATAR_GRADIENTS[id % AVATAR_GRADIENTS.length]
}

function rowDelay(idx: number): number {
  return Math.min(idx, 10)
}
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">
      <PageHeader title="Pazienti" subtitle="Gestisci la tua lista clienti.">
        <button
          type="button"
          class="group relative overflow-hidden text-white font-semibold px-4 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 cursor-pointer focus:outline-none"
          style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
          @click="router.push('/clients/new')"
        >
          <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
          <Plus class="w-4 h-4 relative z-10" />
          <span class="relative z-10">Nuovo Paziente</span>
        </button>
      </PageHeader>

      <!-- Stats strip -->
      <div class="grid grid-cols-3 gap-4 mb-5 animate-in">
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #059669, #047857)">
            <Users class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Pazienti</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.total }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #4f46e5, #4338ca)">
            <Mail class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Con email</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.withEmail }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #d97706, #b45309)">
            <ShieldCheck class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">STS autorizzati</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.stsAuthorized }}</p>
          </div>
        </div>
      </div>

      <!-- Search + count row -->
      <div class="flex items-center gap-3 mb-4 animate-in-d1">
        <div class="relative flex-1">
          <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-sage-400 pointer-events-none" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Cerca per nome o codice fiscale…"
            class="w-full border border-sage-200 rounded-xl pl-10 pr-9 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
            @input="onSearchInput"
          />
          <button
            v-if="searchQuery"
            type="button"
            class="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 text-sage-400 hover:text-sage-600 transition-colors cursor-pointer rounded"
            @click="clearSearch"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>
        <span class="text-xs text-sage-400 whitespace-nowrap shrink-0">
          {{ clientsStore.clients.length }} {{ clientsStore.clients.length === 1 ? 'paziente' : 'pazienti' }}
        </span>
      </div>

      <!-- Table card -->
      <div class="glass-card rounded-2xl shadow-sm overflow-hidden animate-in-d1">

        <!-- Loading -->
        <div v-if="clientsStore.loading" class="flex flex-col items-center justify-center py-16 gap-3">
          <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
          <p class="text-sm text-sage-400">Caricamento pazienti…</p>
        </div>

        <!-- Empty state -->
        <div
          v-else-if="clientsStore.clients.length === 0"
          class="flex flex-col items-center justify-center py-16 text-center px-6"
        >
          <div class="w-14 h-14 rounded-2xl bg-sage-50 flex items-center justify-center mb-3">
            <Users class="w-7 h-7 text-sage-300" />
          </div>
          <p class="text-sm font-semibold text-sage-600">Nessun paziente trovato</p>
          <p class="text-xs text-sage-400 mt-1">
            {{ searchQuery ? 'Prova con un termine diverso.' : 'Aggiungi il primo paziente.' }}
          </p>
          <button
            v-if="!searchQuery"
            type="button"
            class="mt-4 flex items-center gap-1.5 text-sm font-medium text-sage-600 hover:text-sage-800 transition-colors cursor-pointer"
            @click="router.push('/clients/new')"
          >
            <Plus class="w-4 h-4" />
            Aggiungi paziente
          </button>
          <button
            v-else
            type="button"
            class="mt-3 text-xs text-sage-400 hover:text-sage-600 transition-colors cursor-pointer underline underline-offset-2"
            @click="clearSearch"
          >
            Rimuovi filtro
          </button>
        </div>

        <!-- Table -->
        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-sage-100/60 bg-sage-50/30">
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Paziente</th>
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Codice Fiscale</th>
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Contatti</th>
              <th class="px-5 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Città</th>
              <th class="px-5 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Azioni</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-sage-50">
            <tr
              v-for="(client, idx) in clientsStore.clients"
              :key="client.id"
              :style="{ '--i': rowDelay(idx) }"
              class="client-row hover:bg-sage-50/60 transition-all duration-150 cursor-pointer group"
              @click="router.push(`/clients/${client.id}/edit`)"
            >
              <!-- Avatar + Name -->
              <td class="px-5 py-3.5">
                <div class="flex items-center gap-3">
                  <div
                    class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0 text-white text-xs font-bold shadow-sm"
                    :style="{ background: avatarGradient(client.id) }"
                  >
                    {{ initials(client) }}
                  </div>
                  <div class="min-w-0">
                    <div class="flex items-center gap-1.5 flex-wrap">
                      <p class="font-semibold text-sage-900 leading-tight truncate">{{ displayName(client) }}</p>
                      <span
                        v-if="client.sts_authorization"
                        class="inline-flex items-center gap-0.5 text-[9px] font-bold px-1.5 py-0.5 rounded-full bg-amber-50 text-amber-600 border border-amber-100 shrink-0"
                      >
                        <ShieldCheck class="w-2.5 h-2.5" />
                        STS
                      </span>
                    </div>
                    <p class="text-[10px] text-sage-400 mt-0.5 capitalize">
                      {{ client.client_type === 'azienda' ? 'Azienda' : client.gender === 'F' ? 'Donna' : 'Uomo' }}<span
                        v-if="client.birth_date"
                      > · {{ new Date(client.birth_date).getFullYear() }}</span>
                    </p>
                  </div>
                </div>
              </td>

              <!-- Fiscal code -->
              <td class="px-5 py-3.5">
                <span class="font-mono text-xs text-sage-600 bg-sage-50 border border-sage-100 px-2 py-1 rounded-lg">
                  {{ client.fiscal_code || '—' }}
                </span>
              </td>

              <!-- Contacts -->
              <td class="px-5 py-3.5 min-w-[160px]">
                <p class="text-xs text-sage-700 truncate">{{ client.email || '—' }}</p>
                <p class="text-xs text-sage-400 mt-0.5">{{ client.phone || '—' }}</p>
              </td>

              <!-- City -->
              <td class="px-5 py-3.5">
                <div v-if="client.city" class="flex items-center gap-1.5">
                  <MapPin class="w-3 h-3 text-sage-400 shrink-0" />
                  <span class="text-xs text-sage-600 truncate">{{ client.city }}<span v-if="client.province" class="text-sage-400"> ({{ client.province }})</span></span>
                </div>
                <span v-else class="text-xs text-sage-300">—</span>
              </td>

              <!-- Actions: always visible, subtle -->
              <td class="px-5 py-3.5" @click.stop>
                <div class="flex items-center justify-end gap-1">
                  <button
                    type="button"
                    class="p-2 text-sage-300 hover:text-ocean-600 hover:bg-ocean-50 rounded-lg transition-all duration-150 cursor-pointer"
                    title="Modifica"
                    @click="router.push(`/clients/${client.id}/edit`)"
                  >
                    <Pencil class="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    class="p-2 text-sage-300 hover:text-red-500 hover:bg-red-50 rounded-lg transition-all duration-150 cursor-pointer"
                    title="Elimina"
                    @click="confirmDelete(client)"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <ConfirmModal
        :open="!!clientToDelete"
        title="Elimina paziente"
        :message="`Sei sicuro di voler eliminare ${clientToDelete ? displayName(clientToDelete) : ''}?`"
        @confirm="handleDelete"
        @cancel="clientToDelete = null"
      />
    </div>
  </div>
</template>

<style scoped>
.client-row {
  animation: row-in 0.3s ease both;
  animation-delay: calc(var(--i, 0) * 35ms);
}

@keyframes row-in {
  from { opacity: 0; transform: translateX(-6px); }
  to   { opacity: 1; transform: translateX(0); }
}

@media (prefers-reduced-motion: reduce) {
  .client-row { animation: none; }
}
</style>
