/** Patients. */
import { invoke } from '@tauri-apps/api/core'
import type { Client, CreateClientInput, UpdateClientInput } from '@/features/client/model/client'

export const listClients = (search?: string) =>
  invoke<Client[]>('list_clients', { search: search ?? null })
export const getClient = (id: number) => invoke<Client>('get_client', { id })
export const createClient = (input: CreateClientInput) =>
  invoke<Client>('create_client', { input })
export const updateClient = (input: UpdateClientInput) =>
  invoke<Client>('update_client', { input })
export const deleteClient = (id: number) => invoke<void>('delete_client', { id })
