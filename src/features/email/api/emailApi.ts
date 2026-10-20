/** Invoice emails to patients: mailbox, template and sending. */
import { invoke } from '@tauri-apps/api/core'
import type { EmailAccount, EmailConnectionCheck, EmailCredentialsStatus, EmailDraft, EmailPlaceholderInfo, EmailPreview, EmailProviderPreset, EmailTemplate, InvoiceEmail, SendInvoiceEmailInput, UpdateEmailAccountInput } from '@/features/email/model/email'

export const getEmailProviders = () => invoke<EmailProviderPreset[]>('get_email_providers')
export const getEmailAccount = () => invoke<EmailAccount>('get_email_account')
export const updateEmailAccount = (input: UpdateEmailAccountInput) =>
  invoke<EmailAccount>('update_email_account', { input })
export const getEmailCredentialsStatus = () =>
  invoke<EmailCredentialsStatus>('get_email_credentials_status')
export const saveEmailPassword = (password: string) =>
  invoke<EmailCredentialsStatus>('save_email_password', { password })
export const deleteEmailPassword = () => invoke<EmailCredentialsStatus>('delete_email_password')
export const checkEmailConnection = () => invoke<EmailConnectionCheck>('check_email_connection')
export const getEmailTemplate = () => invoke<EmailTemplate>('get_email_template')
export const updateEmailTemplate = (template: EmailTemplate) =>
  invoke<EmailTemplate>('update_email_template', { template })
export const resetEmailTemplate = () => invoke<EmailTemplate>('reset_email_template')
export const getEmailPlaceholders = () => invoke<EmailPlaceholderInfo[]>('get_email_placeholders')
export const previewEmailTemplate = (template: EmailTemplate) =>
  invoke<EmailPreview>('preview_email_template', { template })
export const prepareInvoiceEmail = (invoiceId: number) =>
  invoke<EmailDraft>('prepare_invoice_email', { invoiceId })
export const sendInvoiceEmail = (input: SendInvoiceEmailInput) =>
  invoke<InvoiceEmail>('send_invoice_email', { input })
export const sendPreparedInvoiceEmail = (invoiceId: number) =>
  invoke<InvoiceEmail>('send_prepared_invoice_email', { invoiceId })
export const listInvoiceEmails = (invoiceId?: number) =>
  invoke<InvoiceEmail[]>('list_invoice_emails', { filters: { invoice_id: invoiceId ?? null } })
