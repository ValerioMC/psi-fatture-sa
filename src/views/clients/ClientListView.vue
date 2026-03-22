<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Search, Plus, Pencil, Trash2, Users, Mail, ShieldCheck, MapPin } from 'lucide-vue-next'
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
  'linear-gradient(135deg, #5d8062, #48654c)',
  'linear-gradient(135deg, #0c8aeb, #0153a2)',
  'linear-gradient(135deg, #b88e67, #8a5f42)',
  'linear-gradient(135deg, #7a9b7e, #5d8062)',
  'linear-gradient(135deg, #36a5fa, #0c8aeb)',
  'linear-gradient(135deg, #d4a017, #a16207)',
]

function avatarGradient(id: number): string {
  return AVATAR_GRADIENTS[id % AVATAR_GRADIENTS.length]
}
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">
      <PageHeader title="Pazienti" subtitle="Gestisci la tua lista clienti.">
        <button
          type="button"
          class="relative overflow-hidden bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-all hover:shadow-lg hover:shadow-sage-200"
          @click="router.push('/clients/new')"
        >
          <Plus class="w-4 h-4" />
          Nuovo Paziente
        </button>
      </PageHeader>

      <!-- Stats strip -->
      <div class="grid grid-cols-3 gap-4 mb-5 animate-in">
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #5d8062, #48654c)">
            <Users class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Pazienti</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.total }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #0c8aeb, #0153a2)">
            <Mail class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Con email</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.withEmail }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #d4a017, #a16207)">
            <ShieldCheck class="w-4 h-4 text-white" />
          </div>
          <div>
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">STS autorizzati</p>
            <p class="text-xl font-bold text-sage-900 leading-tight">{{ stats.stsAuthorized }}</p>
          </div>
        </div>
      </div>

      <!-- Search -->
      <div class="relative mb-4 max-w-sm animate-in-d1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-sage-400" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Cerca per nome, codice fiscale..."
          class="w-full border border-sage-200 rounded-xl pl-9 pr-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80"
          @input="onSearchInput"
        />
      </div>

      <!-- Table -->
      <div class="glass-card rounded-2xl shadow-sm animate-in-d1">
        <div v-if="clientsStore.loading" class="flex flex-col items-center justify-center py-16 gap-3">
          <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
          <p class="text-sm text-sage-400">Caricamento...</p>
        </div>

        <div
          v-else-if="clientsStore.clients.length === 0"
          class="flex flex-col items-center justify-center py-16 text-center"
        >
          <div class="w-14 h-14 rounded-2xl bg-sage-50 flex items-center justify-center mb-3">
            <Users class="w-7 h-7 text-sage-300" />
          </div>
          <p class="text-sm font-semibold text-sage-600">Nessun paziente trovato</p>
          <p class="text-xs text-sage-400 mt-1">
            {{ searchQuery ? 'Prova con un termine diverso.' : 'Aggiungi il primo paziente.' }}
          </p>
        </div>

        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-sage-100/60">
              <th class="px-5 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Paziente</th>
              <th class="px-5 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Codice Fiscale</th>
              <th class="px-5 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Contatti</th>
              <th class="px-5 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Città</th>
              <th class="px-5 py-3.5 text-right text-xs font-semibold text-sage-400 uppercase tracking-wider">Azioni</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-sage-50">
            <tr
              v-for="client in clientsStore.clients"
              :key="client.id"
              class="hover:bg-sage-50/40 transition-colors group"
            >
              <!-- Avatar + Name -->
              <td class="px-5 py-3.5">
                <div class="flex items-center gap-3">
                  <div
                    class="w-8 h-8 rounded-full flex items-center justify-center shrink-0 text-white text-xs font-bold"
                    :style="{ background: avatarGradient(client.id) }"
                  >
                    {{ initials(client) }}
                  </div>
                  <div>
                    <p class="font-semibold text-sage-900 leading-tight">{{ displayName(client) }}</p>
                    <p class="text-[10px] text-sage-400 mt-0.5 capitalize">
                      {{ client.client_type === 'azienda' ? 'Azienda' : client.gender === 'F' ? 'Donna' : 'Uomo' }}
                      <span v-if="client.birth_date"> · {{ new Date(client.birth_date).getFullYear() }}</span>
                    </p>
                  </div>
                </div>
              </td>

              <!-- Fiscal code -->
              <td class="px-5 py-3.5">
                <span class="font-mono text-xs text-sage-500 bg-sage-50 px-2 py-0.5 rounded-md">
                  {{ client.fiscal_code || '—' }}
                </span>
              </td>

              <!-- Contacts -->
              <td class="px-5 py-3.5">
                <p class="text-xs text-sage-600">{{ client.email || '—' }}</p>
                <p class="text-xs text-sage-400 mt-0.5">{{ client.phone || '—' }}</p>
              </td>

              <!-- City -->
              <td class="px-5 py-3.5">
                <div v-if="client.city" class="flex items-center gap-1.5">
                  <MapPin class="w-3 h-3 text-sage-400 shrink-0" />
                  <span class="text-xs text-sage-500">{{ client.city }}<span v-if="client.province" class="text-sage-400"> ({{ client.province }})</span></span>
                </div>
                <span v-else class="text-xs text-sage-300">—</span>
              </td>

              <!-- Actions -->
              <td class="px-5 py-3.5">
                <div class="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    type="button"
                    class="p-1.5 text-sage-400 hover:text-ocean-600 hover:bg-ocean-50 rounded-lg transition-colors"
                    title="Modifica"
                    @click="router.push(`/clients/${client.id}/edit`)"
                  >
                    <Pencil class="w-4 h-4" />
                  </button>
                  <button
                    type="button"
                    class="p-1.5 text-sage-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                    title="Elimina"
                    @click="confirmDelete(client)"
                  >
                    <Trash2 class="w-4 h-4" />
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
