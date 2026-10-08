/**
 * The Sistema TS as the app knows it: every local transmission, the settings,
 * and the actions that queue work and send it. Views read an invoice's state
 * through `stateOf`. Every pass reports progress as events and reloads the
 * list when it ends.
 */
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  dispatchTsQueue,
  enqueueTsCancellation,
  enqueueTsReplacement,
  enqueueTsSubmission,
  getTsCredentialsStatus,
  getTsEnvironments,
  getTsSettings,
  listTsSubmissions,
  onTsDispatchProgress,
  updateTsSettings,
  withdrawTsSubmission,
} from '@/api'
import type { TsCredentialsStatus, TsDispatchProgress, TsDispatchSummary, TsEnvironment, TsSettings, TsSubmission } from '@/types'
import { useToastStore } from '@/stores/toast'
import { cancellableSubmissionIds, groupByInvoice, invoiceStsState, type InvoiceStsState } from '@/utils/sts'

export const useStsStore = defineStore('sts', () => {
  const toast = useToastStore()
  const submissions = ref<TsSubmission[]>([])
  const settings = ref<TsSettings | null>(null)
  const environments = ref<TsEnvironment[]>(['produzione'])
  const credentials = ref<TsCredentialsStatus | null>(null)
  const loaded = ref(false)
  const dispatching = ref(false)
  /** The pass in progress, or the last one until a view dismisses it. */
  const progress = ref<TsDispatchProgress | null>(null)
  let progressSubscription: Promise<void> | null = null

  const environment = computed<TsEnvironment>(() => settings.value?.environment ?? 'produzione')
  /** Only a developer build offers the Sogei test environment. */
  const testAvailable = computed(() => environments.value.includes('test'))
  /** Both secrets stored: nothing stops a transmission on this side. */
  const connected = computed(() => credentials.value !== null && credentials.value.password_configured && credentials.value.pincode_configured)
  const byInvoice = computed(() => groupByInvoice(submissions.value))
  const cancellableIds = computed(() => cancellableSubmissionIds(byInvoice.value, environment.value))

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
    const [list, current, secrets, offered] = await Promise.all([
      listTsSubmissions(),
      getTsSettings(),
      getTsCredentialsStatus(),
      getTsEnvironments(),
      subscribeToProgress(),
    ])
    submissions.value = list
    environments.value = offered
    settings.value = current
    credentials.value = secrets
    loaded.value = true
  }

  async function refresh(): Promise<void> {
    submissions.value = await listTsSubmissions()
  }

  /** Subscribes once for the whole session; a failed attempt is forgotten so the next load retries it. */
  function subscribeToProgress(): Promise<void> {
    progressSubscription ??= onTsDispatchProgress(receiveProgress).then(
      () => undefined,
      (error: unknown) => {
        progressSubscription = null
        throw error
      },
    )
    return progressSubscription
  }

  /** Reloads the list where nothing awaits the reload; a failure is reported, the list keeps its last state. */
  async function resync(): Promise<void> {
    try {
      await refresh()
    } catch (error) {
      toast.notifyError(error, 'Elenco del Sistema TS non aggiornato')
    }
  }

  function receiveProgress(report: TsDispatchProgress): void {
    progress.value = report
    if (report.finished) void resync()
  }

  function dismissProgress(): void {
    progress.value = null
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
    credentials,
    connected,
    loaded,
    dispatching,
    progress,
    cancellableIds,
    environment,
    testAvailable,
    counts,
    stateOf,
    load,
    refresh,
    resync,
    saveSettings,
    dispatch,
    send,
    replace,
    cancel,
    withdraw,
    dismissProgress,
  }
})
