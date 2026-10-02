<script setup lang="ts">
/**
 * The patient register. Filtering is local and instant: the whole list is
 * already in memory, and a register of a few hundred names needs no round trip.
 */
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Activity, Building2, ClipboardList, Pencil, Search, ShieldCheck, Trash2, UserPlus, Users } from 'lucide-vue-next'
import { listInvoices } from '@/api'
import { useClientsStore } from '@/stores/clients'
import { useToastStore } from '@/stores/toast'
import type { Client, Invoice } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import StatTile from '@/components/ui/StatTile.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { ageOn, clientDisplayName } from '@/utils/client'
import { plural } from '@/utils/labels'
import { missingFields, patientsInCare, type MissingField } from '@/utils/patientRegister'

const CARE_WINDOW_DAYS = 90

const router = useRouter()
const clientsStore = useClientsStore()
const toast = useToastStore()

const search = ref('')
const onlyIncomplete = ref(false)
const toDelete = ref<Client | null>(null)
const deleting = ref(false)
const invoices = ref<Invoice[]>([])

onMounted(async () => {
  await Promise.all([clientsStore.fetchClients(), loadInvoices()])
  if (clientsStore.error) toast.notifyError(clientsStore.error, 'Caricamento non riuscito')
})

async function loadInvoices(): Promise<void> {
  try {
    invoices.value = await listInvoices({})
  } catch (error) {
    toast.notifyError(error, 'Fatture non caricate')
  }
}

const gapsById = computed(() => new Map<number, MissingField[]>(clientsStore.clients.map((client) => [client.id, missingFields(client)])))
const gapsOf = (client: Client): MissingField[] => gapsById.value.get(client.id) ?? []

function normalise(text: string): string {
  return text.toLocaleLowerCase('it-IT').normalize('NFD').replace(/\p{Diacritic}/gu, '')
}

const sorted = computed(() =>
  [...clientsStore.clients].sort((a, b) => clientDisplayName(a).localeCompare(clientDisplayName(b), 'it')),
)

const visible = computed(() => {
  const needle = normalise(search.value.trim())
  const pool = onlyIncomplete.value ? sorted.value.filter((client) => gapsOf(client).length > 0) : sorted.value
  if (needle === '') return pool
  return pool.filter((client) =>
    normalise(`${clientDisplayName(client)} ${client.fiscal_code} ${client.city} ${client.email ?? ''}`).includes(needle),
  )
})

const subtitle = computed(() => {
  const all = clientsStore.clients
  if (all.length === 0) return undefined
  const sts = all.filter((client) => client.sts_authorization).length
  return `${plural(all.length, 'paziente', 'pazienti')} · ${sts} con consenso al Sistema Tessera Sanitaria`
})

/** The register at a glance: size, consent, who is in care now, and which records block invoicing. */
const stats = computed(() => {
  const all = clientsStore.clients
  const companies = all.filter((client) => client.client_type === 'azienda').length
  const sts = all.filter((client) => client.sts_authorization).length
  const known = new Set(all.map((client) => client.id))
  const inCare = [...patientsInCare(invoices.value, CARE_WINDOW_DAYS)].filter((id) => known.has(id)).length
  const incomplete = all.filter((client) => gapsOf(client).length > 0)
  const countMissing = (field: MissingField): number => incomplete.filter((client) => gapsOf(client).includes(field)).length
  return {
    total: all.length,
    companies,
    sts,
    stsShare: all.length > 0 ? Math.round((sts / all.length) * 100) : 0,
    inCare,
    inCareShare: all.length > 0 ? Math.round((inCare / all.length) * 100) : 0,
    incomplete: incomplete.length,
    missingEmail: countMissing('email'),
    missingForInvoice: incomplete.filter((client) => gapsOf(client).some((field) => field !== 'email')).length,
  }
})

const incompleteHint = computed(() => {
  const { incomplete, missingEmail, missingForInvoice } = stats.value
  if (incomplete === 0) return 'Anagrafiche pronte per fattura ed email'
  const parts: string[] = []
  if (missingForInvoice > 0) parts.push(`${missingForInvoice} senza CF o indirizzo`)
  if (missingEmail > 0) parts.push(`${missingEmail} senza email`)
  return parts.join(' · ')
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
    <PageHeader title="Pazienti" :subtitle="subtitle" :icon="Users">
      <AppButton variant="primary" :icon="UserPlus" to="/clients/new">Nuovo paziente</AppButton>
    </PageHeader>

    <div class="page pt-6 pb-12">
      <div v-if="stats.total > 0" class="settle mb-5 grid grid-cols-2 gap-4 xl:grid-cols-4">
        <StatTile label="Pazienti" :value="String(stats.total)" :icon="Users" tone="accent">
          <template #hint>{{ stats.companies > 0 ? `di cui ${plural(stats.companies, 'azienda o ente', 'aziende o enti')}` : 'Tutte persone fisiche' }}</template>
        </StatTile>
        <StatTile label="Consenso STS" :value="`${stats.stsShare}%`" :icon="ShieldCheck" tone="safe" :hint="`${stats.sts} su ${stats.total} autorizzano la trasmissione`">
          <div class="h-1.5 overflow-hidden rounded-full bg-safe-soft">
            <div class="h-full rounded-full bg-gradient-to-r from-safe/70 to-safe transition-[width] duration-700 ease-out-expo" :style="{ width: `${stats.stsShare}%` }" />
          </div>
        </StatTile>
        <StatTile label="In carico" :value="String(stats.inCare)" :icon="Activity" tone="accent" :hint="`fatturati negli ultimi ${CARE_WINDOW_DAYS} giorni · ${stats.total - stats.inCare} inattivi`">
          <div class="h-1.5 overflow-hidden rounded-full bg-accent-soft">
            <div class="h-full rounded-full bg-gradient-to-r from-accent/70 to-accent transition-[width] duration-700 ease-out-expo" :style="{ width: `${stats.inCareShare}%` }" />
          </div>
        </StatTile>
        <button
          type="button"
          class="min-w-0 rounded-[inherit] text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-default"
          :disabled="stats.incomplete === 0 && !onlyIncomplete"
          :aria-pressed="onlyIncomplete"
          :title="stats.incomplete > 0 ? 'Mostra solo le anagrafiche da completare' : undefined"
          @click="onlyIncomplete = !onlyIncomplete"
        >
          <StatTile
            label="Da completare"
            :value="String(stats.incomplete)"
            :icon="ClipboardList"
            :tone="stats.incomplete > 0 ? 'warn' : 'safe'"
            :hint="incompleteHint"
            class="h-full"
            :class="{ 'ring-2 ring-warn/50': onlyIncomplete }"
          />
        </button>
      </div>

      <div class="settle mb-3 flex items-center gap-3" style="--settle: 1">
        <div class="relative w-full sm:w-80">
          <Search :size="15" class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-text-subtle" aria-hidden="true" />
          <input v-model="search" type="search" class="field field-sm pl-8" placeholder="Nome, codice fiscale, città" aria-label="Cerca pazienti" />
        </div>
        <AppButton v-if="onlyIncomplete" size="sm" @click="onlyIncomplete = false">Mostra tutti</AppButton>
        <span v-if="search || onlyIncomplete" class="text-sm text-text-subtle">{{ plural(visible.length, 'risultato', 'risultati') }}</span>
      </div>

      <AppCard :padded="false" class="settle overflow-hidden" style="--settle: 2">
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

        <EmptyState v-else-if="visible.length === 0" :icon="Search" :bordered="false" :title="search ? `Nessun paziente per “${search}”` : 'Nessuna anagrafica da completare'">
          <AppButton @click="search = ''; onlyIncomplete = false">Azzera i filtri</AppButton>
        </EmptyState>

        <table v-else class="data-table text-base">
          <thead>
            <tr>
              <th class="pl-5 font-medium">Paziente</th>
              <th class="w-48 font-medium">Codice fiscale</th>
              <th class="font-medium">Contatti</th>
              <th class="hidden font-medium 2xl:table-cell">Indirizzo</th>
              <th class="w-44 font-medium">Città</th>
              <th class="hidden w-28 font-medium xl:table-cell">STS</th>
              <th class="w-24 pr-4"><span class="sr-only">Azioni</span></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="client in visible"
              :key="client.id"
              class="group h-14 cursor-pointer"
              data-row
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
                    <span class="flex items-center gap-1 text-xs text-text-subtle">
                      <Building2 v-if="client.client_type === 'azienda'" :size="11" aria-hidden="true" />{{ describe(client) }}
                      <span v-if="gapsOf(client).length > 0" class="text-warn">· manca {{ gapsOf(client).join(', ') }}</span>
                    </span>
                  </span>
                </span>
              </td>
              <td class="font-mono text-sm tracking-[0.02em] text-text-muted">{{ client.fiscal_code || '—' }}</td>
              <td class="min-w-0 max-w-[16rem]">
                <span class="block truncate text-sm text-text-muted">{{ client.email || '—' }}</span>
                <span class="tabular block text-xs text-text-subtle">{{ client.phone }}</span>
              </td>
              <td class="hidden max-w-[18rem] truncate text-sm text-text-muted 2xl:table-cell">{{ client.address || '—' }}</td>
              <td class="text-sm text-text-muted">
                {{ client.city || '—' }}<span v-if="client.province" class="text-text-subtle"> ({{ client.province }})</span>
              </td>
              <td class="hidden xl:table-cell">
                <AppBadge v-if="client.sts_authorization" tone="safe" dot>Autorizza</AppBadge>
                <AppBadge v-else tone="neutral" dot>Non autorizza</AppBadge>
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
