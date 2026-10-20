/** The Sistema Tessera Sanitaria: settings, credentials, queue and remote queries. */
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { TsConnectionCheck, TsCredentialsStatus, TsDispatchProgress, TsDispatchSummary, TsEnvironment, TsQueryResult, TsReportBasis, TsReportRow, TsSettings, TsSubmission, TsSubmissionFilters } from '@/features/sts/model/ts'

/** Production alone in a distributed build; test too in a developer one. */
export const getTsEnvironments = () => invoke<TsEnvironment[]>('get_ts_environments')
export const getTsSettings = () => invoke<TsSettings>('get_ts_settings')
export const updateTsSettings = (input: TsSettings) =>
  invoke<TsSettings>('update_ts_settings', { input })
export const getTsCredentialsStatus = () =>
  invoke<TsCredentialsStatus>('get_ts_credentials_status')
export const saveTsPincode = (pincode: string) =>
  invoke<TsCredentialsStatus>('save_ts_pincode', { pincode })
export const deleteTsPincode = () => invoke<TsCredentialsStatus>('delete_ts_pincode')
export const saveTsPassword = (password: string) =>
  invoke<TsCredentialsStatus>('save_ts_password', { password })
export const deleteTsPassword = () => invoke<TsCredentialsStatus>('delete_ts_password')
export const checkTsConnection = () => invoke<TsConnectionCheck>('check_ts_connection')
export const listTsSubmissions = (filters: TsSubmissionFilters = {}) =>
  invoke<TsSubmission[]>('list_ts_submissions', { filters })
export const enqueueTsSubmission = (invoiceId: number) =>
  invoke<TsSubmission>('enqueue_ts_submission', { invoiceId })
export const isTsInvoiceOutOfDate = (invoiceId: number) =>
  invoke<boolean>('is_ts_invoice_out_of_date', { invoiceId })
export const enqueueTsReplacement = (submissionId: number) =>
  invoke<TsSubmission>('enqueue_ts_replacement', { submissionId })
export const enqueueTsCancellation = (submissionId: number) =>
  invoke<TsSubmission>('enqueue_ts_cancellation', { submissionId })
export const withdrawTsSubmission = (submissionId: number) =>
  invoke<void>('withdraw_ts_submission', { submissionId })
export const dispatchTsQueue = () => invoke<TsDispatchSummary>('dispatch_ts_queue')
/** Every pass over the queue, manual or from the background worker, reports here as it goes. */
export const onTsDispatchProgress = (handler: (progress: TsDispatchProgress) => void): Promise<UnlistenFn> =>
  listen<TsDispatchProgress>('ts-dispatch-progress', (event) => handler(event.payload))
export const queryTsInvoice = (invoiceId: number) =>
  invoke<TsQueryResult>('query_ts_invoice', { invoiceId })
export const getTsMonthlyReport = (year: number, month: number, basis: TsReportBasis) =>
  invoke<TsReportRow[]>('get_ts_monthly_report', { year, month, basis })
