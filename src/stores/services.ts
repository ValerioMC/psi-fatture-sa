import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listServices, createService, updateService, deleteService } from '@/api'
import type { Service, CreateServiceInput, UpdateServiceInput } from '@/types'

export const useServicesStore = defineStore('services', () => {
  const services = ref<Service[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchServices(activeOnly = false) {
    loading.value = true
    error.value = null
    try {
      services.value = await listServices(activeOnly)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addService(input: CreateServiceInput) {
    const service = await createService(input)
    services.value.push(service)
    return service
  }

  async function editService(input: UpdateServiceInput) {
    const updated = await updateService(input)
    const idx = services.value.findIndex((s) => s.id === updated.id)
    if (idx !== -1) services.value[idx] = updated
    return updated
  }

  async function removeService(id: number) {
    await deleteService(id)
    services.value = services.value.filter((s) => s.id !== id)
  }

  return { services, loading, error, fetchServices, addService, editService, removeService }
})
