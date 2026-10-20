/** Acceptance of the terms of use. */
import { invoke } from '@tauri-apps/api/core'
import type { AcceptTermsInput, TermsAcceptance } from '@/features/terms/model/terms'

export const getTermsAcceptance = (version: string) =>
  invoke<TermsAcceptance | null>('get_terms_acceptance', { version })
export const acceptTerms = (input: AcceptTermsInput) => invoke<TermsAcceptance>('accept_terms', { input })
