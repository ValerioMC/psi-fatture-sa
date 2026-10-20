/** The catalogue of services offered. */
import { invoke } from '@tauri-apps/api/core'
import type { CreateServiceInput, Service, UpdateServiceInput } from '@/features/service/model/service'

export const listServices = (activeOnly = false) =>
  invoke<Service[]>('list_services', { activeOnly })
export const createService = (input: CreateServiceInput) =>
  invoke<Service>('create_service', { input })
export const updateService = (input: UpdateServiceInput) =>
  invoke<Service>('update_service', { input })
export const deleteService = (id: number) => invoke<void>('delete_service', { id })
