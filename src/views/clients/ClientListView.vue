<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Search, Plus, Pencil, Trash2, Users } from 'lucide-vue-next'
import { useClientsStore } from '@/stores/clients'
import type { Client } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import ConfirmModal from '@/components/ui/ConfirmModal.vue'

const router = useRouter()
const clientsStore = useClientsStore()

const searchQuery = ref('')
const clientToDelete = ref<Client | null>(null)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

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

function clientTypeLabel(type: string): string {
  return type === 'azienda' ? 'Azienda' : 'Persona Fisica'
}
</script>

<template>
  <div class="p-8">
    <PageHeader title="Clienti" subtitle="Gestisci i tuoi pazienti e clienti.">
      <button
        type="button"
        class="relative overflow-hidden bg-gradient-to-r from-sage-600 to-ocean-500 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-all hover:shadow-lg hover:shadow-sage-200"
        @click="router.push('/clients/new')"
      >
        <Plus class="w-4 h-4" />
        Nuovo Cliente
      </button>
    </PageHeader>

    <!-- Search -->
    <div class="relative mb-4 max-w-sm animate-in">
      <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-sage-400" />
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Cerca per nome, cognome, codice fiscale..."
        class="w-full border border-sage-200 rounded-lg pl-9 pr-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
        @input="onSearchInput"
      />
    </div>

    <!-- Table -->
    <div class="glass-card rounded-xl animate-in-d1">
      <div v-if="clientsStore.loading" class="px-6 py-12 text-center text-sm text-sage-400">
        Caricamento...
      </div>

      <div
        v-else-if="clientsStore.clients.length === 0"
        class="flex flex-col items-center justify-center py-16 text-center"
      >
        <Users class="w-12 h-12 text-sage-200 mb-3" />
        <p class="text-sm font-medium text-sage-500">Nessun cliente trovato</p>
        <p class="text-xs text-sage-400 mt-1">
          {{ searchQuery ? 'Prova con un termine di ricerca diverso.' : 'Aggiungi il primo cliente.' }}
        </p>
      </div>

      <table v-else class="w-full text-sm">
        <thead>
          <tr class="border-b border-sage-100">
            <th class="px-6 py-3 text-left text-xs font-medium text-sage-500 uppercase tracking-wider">Nome</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-sage-500 uppercase tracking-wider">Tipo</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-sage-500 uppercase tracking-wider">Codice Fiscale</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-sage-500 uppercase tracking-wider">Email</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-sage-500 uppercase tracking-wider">Telefono</th>
            <th class="px-6 py-3 text-right text-xs font-medium text-sage-500 uppercase tracking-wider">Azioni</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-sage-100/50">
          <tr
            v-for="client in clientsStore.clients"
            :key="client.id"
            class="hover:bg-sage-50/40 transition-colors"
          >
            <td class="px-6 py-3 font-medium text-sage-900">{{ displayName(client) }}</td>
            <td class="px-6 py-3 text-sage-500">{{ clientTypeLabel(client.client_type) }}</td>
            <td class="px-6 py-3 text-sage-500 font-mono text-xs">{{ client.fiscal_code }}</td>
            <td class="px-6 py-3 text-sage-500">{{ client.email ?? '—' }}</td>
            <td class="px-6 py-3 text-sage-500">{{ client.phone || '—' }}</td>
            <td class="px-6 py-3">
              <div class="flex items-center justify-end gap-2">
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
      title="Elimina cliente"
      :message="`Sei sicuro di voler eliminare ${clientToDelete ? displayName(clientToDelete) : ''}? Questa azione non può essere annullata.`"
      @confirm="handleDelete"
      @cancel="clientToDelete = null"
    />
  </div>
</template>
