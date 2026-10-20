export type EmailProvider = 'psypec' | 'aruba_pec' | 'gmail' | 'custom'

export type EmailSecurity = 'tls' | 'starttls'

export type InvoiceEmailStatus = 'sent' | 'failed'

/** A provider's SMTP settings; `certified` marks a PEC mailbox. */
export interface EmailProviderPreset {
  provider: EmailProvider
  label: string
  host: string
  port: number
  security: EmailSecurity
  domains: string[]
  certified: boolean
  note: string
}

/** The mailbox invoices leave from; `saved` is false while it is only proposed from the profile. */
export interface EmailAccount {
  provider: EmailProvider
  sender_address: string
  sender_name: string
  smtp_host: string
  smtp_port: number
  security: EmailSecurity
  username: string
  bcc_self: boolean
  saved: boolean
}

export type UpdateEmailAccountInput = Omit<EmailAccount, 'saved'>

export interface EmailCredentialsStatus {
  password_configured: boolean
}

export interface EmailConnectionCheck {
  ok: boolean
  message: string
}

export interface EmailTemplate {
  subject: string
  body: string
}

export interface EmailPlaceholderInfo {
  key: string
  label: string
}

export type EmailPreview = EmailTemplate

/** The email proposed for one invoice; `recipient_on_file` is false when the patient has no address. */
export interface EmailDraft {
  invoice_id: number
  recipient: string
  recipient_on_file: boolean
  subject: string
  body: string
  attachment_name: string
}

export interface SendInvoiceEmailInput {
  invoice_id: number
  recipient: string
  subject: string
  body: string
  remember_recipient: boolean
}

/** One attempt to email an invoice; `sent_at` is UTC, "YYYY-MM-DD HH:MM:SS". */
export interface InvoiceEmail {
  id: number
  invoice_id: number
  recipient: string
  subject: string
  attachment_name: string
  status: InvoiceEmailStatus
  error: string | null
  sent_at: string
}
