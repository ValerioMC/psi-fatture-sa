<script setup lang="ts">
/**
 * Every transmission, newest first: what went out, what the Sistema TS
 * answered, and the one action each row still allows. A filter narrows to
 * what needs attention.
 */
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { Ban, History, RotateCcw, Undo2 } from 'lucide-vue-next'
import type { TsSubmission, TsSubmissionStatus } from '@/types'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import type { SegmentOption } from '@/components/ui/types'
import { TS_OPERATION_LABEL, TS_SUBMISSION_STATUS, formatStsTimestamp } from '@/utils/sts'

type Filter = 'all' | 'queue' | 'accepted' | 'rejected'

const props = defineProps<{
  submissions: readonly TsSubmission[]
  /** Accepted sends that are still the live data of their invoice. */
  liveIds: ReadonlySet<number>
  busyId: number | null
}>()

const emit = defineEmits<{
  withdraw: [submission: TsSubmission]
  cancel: [submission: TsSubmission]
  resend: [submission: TsSubmission]
}>()

const filter = ref<Filter>('all')

const GROUP: Record<Exclude<Filter, 'all'>, readonly TsSubmissionStatus[]> = {
  queue: ['non_inviata', 'inviata'],
  accepted: ['accettata'],
  rejected: ['scartata'],
}

const options = computed<SegmentOption<Filter>[]>(() => {
  const count = (group: Exclude<Filter, 'all'>) => props.submissions.filter((s) => GROUP[group].includes(s.status)).length
  return [
    { value: 'all', label: 'Tutte', count: props.submissions.length },
    { value: 'queue', label: 'In coda', count: count('queue'), tone: 'neutral' },
    { value: 'accepted', label: 'Accettate', count: count('accepted'), tone: 'safe' },
    { value: 'rejected', label: 'Scartate', count: count('rejected'), tone: 'danger' },
  ]
})

const visible = computed(() =>
  [...props.submissions]
    .filter((s) => filter.value === 'all' || GROUP[filter.value].includes(s.status))
    .sort((a, b) => b.id - a.id),
)

/** A rejected first send is retried by sending the invoice again. */
function canResend(submission: TsSubmission): boolean {
  if (submission.status !== 'scartata' || submission.operation !== 'invio') return false
  const newer = props.submissions.some((s) => s.invoice_id === submission.invoice_id && s.id > submission.id)
  return !newer
}
</script>

<template>
  <div class="flex items-center gap-3 border-b border-border px-5 py-3">
    <SegmentedControl v-model="filter" :options="options" label="Filtra le trasmissioni" size="sm" />
  </div>

  <EmptyState
    v-if="visible.length === 0"
    :icon="History"
    :bordered="false"
    :title="submissions.length === 0 ? 'Ancora nessuna trasmissione' : 'Nessuna trasmissione con questo esito'"
    :description="submissions.length === 0 ? 'Le fatture pagate che trasmetti compaiono qui, con l’esito del Sistema TS.' : undefined"
  />

  <table v-else class="data-table text-base">
    <thead>
      <tr>
        <th class="w-32 pl-5 font-medium">Esito</th>
        <th class="w-32 font-medium">Operazione</th>
        <th class="w-28 font-medium">Fattura</th>
        <th class="font-medium">Paziente</th>
        <th class="hidden w-48 font-medium xl:table-cell">Protocollo</th>
        <th class="w-32 font-medium">Quando</th>
        <th class="w-36 pr-4"><span class="sr-only">Azioni</span></th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="submission in visible" :key="submission.id" class="group h-row align-middle">
        <td class="pl-5">
          <AppBadge :tone="TS_SUBMISSION_STATUS[submission.status].tone" dot>{{ TS_SUBMISSION_STATUS[submission.status].label }}</AppBadge>
        </td>
        <td class="text-sm text-text-muted">
          {{ TS_OPERATION_LABEL[submission.operation] }}
          <AppBadge v-if="submission.environment === 'test'" tone="warn" mono class="ml-1">TEST</AppBadge>
        </td>
        <td class="tabular text-text-muted">
          <RouterLink :to="`/invoices/${submission.invoice_id}`" class="rounded-sm hover:text-text hover:underline focus-ring">
            N. {{ submission.invoice_number }}/{{ submission.invoice_year }}
          </RouterLink>
        </td>
        <td class="max-w-0">
          <p class="truncate text-text">{{ submission.client_name }}</p>
          <p
            v-if="submission.outcome_message || submission.last_error"
            class="truncate text-xs"
            :class="submission.status === 'scartata' ? 'text-danger' : 'text-text-subtle'"
            :title="submission.outcome_message ?? submission.last_error ?? undefined"
          >
            {{ submission.status === 'non_inviata' ? submission.last_error : submission.outcome_message }}
          </p>
        </td>
        <td class="hidden font-mono text-xs text-text-muted xl:table-cell">{{ submission.protocol ?? '—' }}</td>
        <td class="tabular text-sm text-text-subtle">{{ formatStsTimestamp(submission.resolved_at ?? submission.last_attempt_at ?? submission.created_at) }}</td>
        <td class="pr-4">
          <div class="flex justify-end">
            <AppButton
              v-if="submission.status === 'non_inviata'"
              variant="ghost"
              size="sm"
              :icon="Undo2"
              :loading="busyId === submission.id"
              @click="emit('withdraw', submission)"
            >
              Ritira
            </AppButton>
            <AppButton
              v-else-if="liveIds.has(submission.id)"
              variant="danger-quiet"
              size="sm"
              :icon="Ban"
              :loading="busyId === submission.id"
              @click="emit('cancel', submission)"
            >
              Annulla
            </AppButton>
            <AppButton
              v-else-if="canResend(submission)"
              variant="ghost"
              size="sm"
              :icon="RotateCcw"
              :loading="busyId === submission.id"
              @click="emit('resend', submission)"
            >
              Ritrasmetti
            </AppButton>
          </div>
        </td>
      </tr>
    </tbody>
  </table>
</template>
