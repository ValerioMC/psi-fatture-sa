/**
 * The mailbox invoices leave from, and every attempt to email one. Views read an
 * invoice's state through `stateOf`, as they do for the Sistema TS.
 */
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  deleteEmailPassword,
  getEmailAccount,
  getEmailCredentialsStatus,
  getEmailProviders,
  listInvoiceEmails,
  saveEmailPassword,
  sendInvoiceEmail,
  sendPreparedInvoiceEmail,
  updateEmailAccount,
} from '@/api'
import type {
  EmailAccount,
  EmailCredentialsStatus,
  EmailProviderPreset,
  InvoiceEmail,
  SendInvoiceEmailInput,
  UpdateEmailAccountInput,
} from '@/types'
import { groupEmailsByInvoice, invoiceEmailState, isEmailReady, type InvoiceEmailState } from '@/utils/email'

export const useEmailStore = defineStore('email', () => {
  const providers = ref<EmailProviderPreset[]>([])
  const account = ref<EmailAccount | null>(null)
  const credentials = ref<EmailCredentialsStatus | null>(null)
  const emails = ref<InvoiceEmail[]>([])
  const loaded = ref(false)

  const ready = computed(() => isEmailReady(account.value, credentials.value))
  const byInvoice = computed(() => groupEmailsByInvoice(emails.value))
  const preset = computed(() => providers.value.find((p) => p.provider === account.value?.provider) ?? null)

  function stateOf(invoiceId: number): InvoiceEmailState {
    return invoiceEmailState(byInvoice.value.get(invoiceId) ?? [])
  }

  async function load(): Promise<void> {
    const [offered, current, secrets, list] = await Promise.all([getEmailProviders(), getEmailAccount(), getEmailCredentialsStatus(), listInvoiceEmails()])
    providers.value = offered
    account.value = current
    credentials.value = secrets
    emails.value = list
    loaded.value = true
  }

  async function refresh(): Promise<void> {
    emails.value = await listInvoiceEmails()
  }

  async function saveAccount(input: UpdateEmailAccountInput): Promise<EmailAccount> {
    account.value = await updateEmailAccount(input)
    return account.value
  }

  async function savePassword(password: string): Promise<void> {
    credentials.value = await saveEmailPassword(password)
  }

  async function removePassword(): Promise<void> {
    credentials.value = await deleteEmailPassword()
  }

  /** A failure is recorded by the backend too, so the list is reloaded before rethrowing. */
  async function recorded(attempt: () => Promise<InvoiceEmail>): Promise<InvoiceEmail> {
    try {
      const record = await attempt()
      emails.value = [record, ...emails.value]
      return record
    } catch (error) {
      await refresh().catch(() => undefined)
      throw error
    }
  }

  function send(input: SendInvoiceEmailInput): Promise<InvoiceEmail> {
    return recorded(() => sendInvoiceEmail(input))
  }

  function sendPrepared(invoiceId: number): Promise<InvoiceEmail> {
    return recorded(() => sendPreparedInvoiceEmail(invoiceId))
  }

  return {
    providers,
    account,
    credentials,
    emails,
    loaded,
    ready,
    preset,
    stateOf,
    load,
    refresh,
    saveAccount,
    savePassword,
    removePassword,
    send,
    sendPrepared,
  }
})
