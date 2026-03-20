import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listClients, createClient, updateClient, deleteClient } from '@/api'
import type { Client, CreateClientInput, UpdateClientInput } from '@/types'

export const useClientsStore = defineStore('clients', () => {
  const clients = ref<Client[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchClients(search?: string) {
    loading.value = true
    error.value = null
    try {
      clients.value = await listClients(search)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addClient(input: CreateClientInput) {
    const client = await createClient(input)
    clients.value.push(client)
    return client
  }

  async function editClient(input: UpdateClientInput) {
    const updated = await updateClient(input)
    const idx = clients.value.findIndex((c) => c.id === updated.id)
    if (idx !== -1) clients.value[idx] = updated
    return updated
  }

  async function removeClient(id: number) {
    await deleteClient(id)
    clients.value = clients.value.filter((c) => c.id !== id)
  }

  return { clients, loading, error, fetchClients, addClient, editClient, removeClient }
})
