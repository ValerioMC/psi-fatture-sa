export type TsEnvironment = 'test' | 'produzione'

export type TsOperation = 'invio' | 'sostituzione' | 'annullamento'

export type TsSubmissionStatus =
  | 'non_inviata'
  | 'inviata'
  | 'accettata'
  | 'scartata'
  | 'annullata'
  | 'sostituita'

/** How the Sistema TS identifies a document: issuer P.IVA, issue date, number. */
export interface TsDocumentId {
  vat_number: string
  issue_date: string
  number: string
}

export interface TsSubmission {
  id: number
  invoice_id: number
  invoice_number: string
  invoice_year: number
  client_name: string
  operation: TsOperation
  status: TsSubmissionStatus
  target_submission_id: number | null
  environment: TsEnvironment
  document: TsDocumentId | null
  protocol: string | null
  outcome_code: string | null
  outcome_message: string | null
  attempt_count: number
  last_error: string | null
  next_attempt_at: string
  last_attempt_at: string | null
  sent_at: string | null
  resolved_at: string | null
  created_at: string
  updated_at: string
}

export interface TsSubmissionFilters {
  invoice_id?: number
  year?: number
  status?: TsSubmissionStatus
}

/** Non-secret settings: `username` is the codice fiscale used to log in. */
export interface TsSettings {
  environment: TsEnvironment
  username: string
  vat_number: string
}

/** Whether the secrets are stored; their values never reach the UI. */
export interface TsCredentialsStatus {
  password_configured: boolean
  pincode_configured: boolean
}

export interface TsConnectionCheck {
  ok: boolean
  message: string
}

export interface TsDispatchSummary {
  accepted: number
  rejected: number
  retrying: number
  waiting_other_environment: number
  blocked: string | null
}

/** Where a pass over the queue stands; `finished` is its last report, also when it stopped early. */
export interface TsDispatchProgress {
  total: number
  processed: number
  finished: boolean
  summary: TsDispatchSummary
}

export interface TsMessage {
  code: string
  description: string
  kind: string
}

export interface TsExpenseTotal {
  expense_type: string
  amount: number
}

export interface TsRemoteDocument {
  id: TsDocumentId
  payment_date: string | null
  totals: TsExpenseTotal[]
  refunded_totals: TsExpenseTotal[]
  protocol: string | null
  sent_date: string | null
  send_kind: string | null
  cancelled: boolean
  messages: TsMessage[]
}

export type TsQueryResult =
  | { kind: 'found'; document: TsRemoteDocument }
  | { kind: 'not_found' }
  | { kind: 'refused'; messages: TsMessage[] }

export type TsReportBasis = 'invio' | 'pagamento'

export interface TsReportRow {
  vat_number: string
  issue_date: string
  document_number: string
  payment_date: string
  protocol: string
  sent_date: string
  send_kind: string
  amount: number
  refunded_amount: number
  invoice_id: number | null
}
