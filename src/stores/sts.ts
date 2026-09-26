/**
 * The Sistema TS as the app knows it: every local transmission, the
 * settings, and the actions that queue work and then send it right away.
 * Views read one invoice's state through `stateOf`.
 */
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  dispatchTsQueue,
  enqueueTsCancellation,
  enqueueTsReplacement,
  enqueueTsSubmission,
  getTsSettings,
  listTsSubmissions,
  updateTsSettings,
  withdrawTsSubmission,
} from '@/api'
import type { TsDispatchSummary, TsEnvironment, TsSettings, TsSubmission } from '@/types'
import { groupByInvoice, invoiceStsState, type InvoiceStsState } from '@/utils/sts'

export const useStsStore = defineStore('sts', () => {
  const submissions = ref<TsSubmission[]>([])
  const settings = ref<TsSettings | null>(null)
  const loaded = ref(false)
  const dispatching = ref(false)

  const environment = computed<TsEnvironment>(() => settings.value?.environment ?? 'produzione')
  const byInvoice = computed(() => groupByInvoice(submissions.value))

  function stateOf(invoiceId: number): InvoiceStsState {
    return invoiceStsState(byInvoice.value.get(invoiceId) ?? [], environment.value)
  }

  /** Invoice-level counts for the sidebar and the Sistema TS page. */
  const counts = computed(() => {
    let queued = 0
    let rejected = 0
    let accepted = 0
    for (const invoiceId of byInvoice.value.keys()) {
      const kind = stateOf(invoiceId).kind
      if (kind === 'queued' || kind === 'sending') queued += 1
      else if (kind === 'rejected') rejected += 1
      else if (kind === 'accepted') accepted += 1
    }
    return { queued, rejected, accepted }
  })

  async function load(): Promise<void> {
    const [list, current] = await Promise.all([listTsSubmissions(), getTsSettings()])
    submissions.value = list
    settings.value = current
    loaded.value = true
  }

  async function refresh(): Promise<void> {
    submissions.value = await listTsSubmissions()
  }

  async function saveSettings(input: TsSettings): Promise<TsSettings> {
    settings.value = await updateTsSettings(input)
    return settings.value
  }

  /** Sends what is due now; the background worker does the same every minute. */
  async function dispatch(): Promise<TsDispatchSummary> {
    dispatching.value = true
    try {
      return await dispatchTsQueue()
    } finally {
      await refresh()
      dispatching.value = false
    }
  }

  /** Queues each invoice, then sends. Failures to queue come back per invoice. */
  async function send(invoiceIds: readonly number[]): Promise<{ summary: TsDispatchSummary; refused: { invoiceId: number; reason: unknown }[] }> {
    const refused: { invoiceId: number; reason: unknown }[] = []
    for (const invoiceId of invoiceIds) {
      try {
        await enqueueTsSubmission(invoiceId)
      } catch (reason) {
        refused.push({ invoiceId, reason })
      }
    }
    const summary = await dispatch()
    return { summary, refused }
  }

  async function replace(submissionId: number): Promise<TsDispatchSummary> {
    await enqueueTsReplacement(submissionId)
    return dispatch()
  }

  async function cancel(submissionId: number): Promise<TsDispatchSummary> {
    await enqueueTsCancellation(submissionId)
    return dispatch()
  }

  async function withdraw(submissionId: number): Promise<void> {
    await withdrawTsSubmission(submissionId)
    await refresh()
  }

  return {
    submissions,
    settings,
    loaded,
    dispatching,
    environment,
    counts,
    stateOf,
    load,
    refresh,
    saveSettings,
    dispatch,
    send,
    replace,
    cancel,
    withdraw,
  }
})
