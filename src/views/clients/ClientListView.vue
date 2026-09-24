<script setup lang="ts">
/**
 * The patient register. Filtering is local and instant: the whole list is
 * already in memory, and a register of a few hundred names needs no round trip.
 */
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Pencil, Search, ShieldCheck, Trash2, UserPlus, Users } from 'lucide-vue-next'
import { useClientsStore } from '@/stores/clients'
import { useToastStore } from '@/stores/toast'
import type { Client } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { ageOn, clientDisplayName } from '@/utils/client'
import { plural } from '@/utils/labels'

const router = useRouter()
const clientsStore = useClientsStore()
const toast = useToastStore()

const search = ref('')
const toDelete = ref<Client | null>(null)
const deleting = ref(false)

onMounted(async () => {
  await clientsStore.fetchClients()
  if (clientsStore.error) toast.notifyError(clientsStore.error, 'Caricamento non riuscito')
})

function normalise(text: string): string {
  return text.toLocaleLowerCase('it-IT').normalize('NFD').replace(/\p{Diacritic}/gu, '')
}

const sorted = computed(() =>
  [...clientsStore.clients].sort((a, b) => clientDisplayName(a).localeCompare(clientDisplayName(b), 'it')),
)

const visible = computed(() => {
  const needle = normalise(search.value.trim())
  if (needle === '') return sorted.value
  return sorted.value.filter((client) =>
    normalise(`${clientDisplayName(client)} ${client.fiscal_code} ${client.city} ${client.email ?? ''}`).includes(needle),
  )
})

const subtitle = computed(() => {
  const all = clientsStore.clients
  if (all.length === 0) return undefined
  const sts = all.filter((client) => client.sts_authorization).length
  return `${plural(all.length, 'paziente', 'pazienti')} · ${sts} con consenso al Sistema Tessera Sanitaria`
})

function describe(client: Client): string {
  if (client.client_type === 'azienda') return 'Azienda o ente'
  const age = ageOn(client.birth_date)
  return age === null ? 'Persona fisica' : `${age} anni`
}

async function confirmDelete(): Promise<void> {
  const client = toDelete.value
  if (client === null) return
  deleting.value = true
  try {
    await clientsStore.removeClient(client.id)
    toast.notify(`${clientDisplayName(client)} eliminato dall'anagrafica`)
    toDelete.value = null
  } catch (error) {
    toast.notifyError(error, 'Eliminazione non riuscita')
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div>
    <PageHeader title="Pazienti" :subtitle="subtitle">
      <AppButton variant="primary" :icon="UserPlus" to="/clients/new">Nuovo paziente</AppButton>
    </PageHeader>

    <div class="mx-auto max-w-[72rem] px-8 pt-6 pb-12">
      <div class="settle mb-3 flex items-center gap-3">
        <div class="relative w-80">
          <Search :size="15" class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-text-subtle" aria-hidden="true" />
          <input v-model="search" type="search" class="field field-sm pl-8" placeholder="Nome, codice fiscale, città" aria-label="Cerca pazienti" />
        </div>
        <span v-if="search" class="text-sm text-text-subtle">{{ plural(visible.length, 'risultato', 'risultati') }}</span>
      </div>

      <AppCard :padded="false" class="settle overflow-hidden" style="--settle: 1">
        <SkeletonRows v-if="clientsStore.loading" variant="table" :count="8" label="Caricamento dei pazienti" />

        <EmptyState
          v-else-if="clientsStore.clients.length === 0"
          :icon="Users"
          :bordered="false"
          title="Ancora nessun paziente"
          description="Aggiungi il primo: servono nome, codice fiscale e indirizzo per poterlo fatturare."
        >
          <AppButton variant="primary" :icon="UserPlus" to="/clients/new">Nuovo paziente</AppButton>
        </EmptyState>

        <EmptyState v-else-if="visible.length === 0" :icon="Search" :bordered="false" :title="`Nessun paziente per “${search}”`">
          <AppButton @click="search = ''">Azzera la ricerca</AppButton>
        </EmptyState>

        <table v-else class="w-full text-base">
          <thead class="border-b border-border bg-surface text-left text-xs text-text-subtle">
            <tr class="h-9">
              <th class="pl-5 font-medium">Paziente</th>
              <th class="w-48 font-medium">Codice fiscale</th>
              <th class="font-medium">Contatti</th>
              <th class="w-40 font-medium">Città</th>
              <th class="w-24 pr-4"><span class="sr-only">Azioni</span></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="client in visible"
              :key="client.id"
              class="group h-14 cursor-pointer border-b border-border last:border-b-0 transition-colors hover:bg-surface-hover"
              @click="router.push(`/clients/${client.id}/edit`)"
            >
              <td class="pl-5">
                <span class="flex min-w-0 items-center gap-3">
                  <PatientMonogram :name="clientDisplayName(client)" />
                  <span class="min-w-0">
                    <span class="flex items-center gap-1.5">
                      <span class="truncate font-medium text-text">{{ clientDisplayName(client) }}</span>
                      <ShieldCheck
                        v-if="client.sts_authorization"
                        :size="14"
                        :stroke-width="1.9"
                        class="shrink-0 text-safe"
                        aria-label="Consenso al Sistema Tessera Sanitaria"
                      />
                    </span>
                    <span class="block text-xs text-text-subtle">{{ describe(client) }}</span>
                  </span>
                </span>
              </td>
              <td class="font-mono text-sm tracking-[0.02em] text-text-muted">{{ client.fiscal_code || '—' }}</td>
              <td class="min-w-0 max-w-[16rem]">
                <span class="block truncate text-sm text-text-muted">{{ client.email || '—' }}</span>
                <span class="tabular block text-xs text-text-subtle">{{ client.phone }}</span>
              </td>
              <td class="text-sm text-text-muted">
                {{ client.city || '—' }}<span v-if="client.province" class="text-text-subtle"> ({{ client.province }})</span>
              </td>
              <td class="pr-4" @click.stop>
                <div class="flex justify-end gap-0.5 opacity-60 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
                  <AppButton variant="ghost" size="sm" :icon="Pencil" :label="`Modifica ${clientDisplayName(client)}`" :to="`/clients/${client.id}/edit`" />
                  <AppButton variant="danger-quiet" size="sm" :icon="Trash2" :label="`Elimina ${clientDisplayName(client)}`" @click="toDelete = client" />
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </AppCard>
    </div>

    <ConfirmDialog
      :open="toDelete !== null"
      title="Eliminare il paziente?"
      message="L'operazione non si può annullare."
      :blast-radius="toDelete ? `${clientDisplayName(toDelete)} verrà tolto dall'anagrafica. Se ha fatture o appuntamenti, l'eliminazione verrà rifiutata per non lasciarli senza intestatario.` : undefined"
      :loading="deleting"
      @confirm="confirmDelete"
      @cancel="toDelete = null"
    />
  </div>
</template>
