/** Whether the current version of the terms of use is accepted; the router asks before every page. */
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { acceptTerms, getTermsAcceptance } from '@/api'
import { TERMS_VERSION } from '@/legal/terms'
import type { TermsAcceptance } from '@/types'

export interface TermsChoice {
  termsAccepted: boolean
  clausesApproved: boolean
}

export const useTermsStore = defineStore('terms', () => {
  const acceptance = ref<TermsAcceptance | null>(null)
  const loaded = ref(false)

  const accepted = computed(() => acceptance.value !== null)

  async function load(): Promise<void> {
    acceptance.value = await getTermsAcceptance(TERMS_VERSION)
    loaded.value = true
  }

  /** The backend refuses unless both boxes are ticked, so the record always holds the specific approval. */
  async function accept(choice: TermsChoice): Promise<TermsAcceptance> {
    acceptance.value = await acceptTerms({
      version: TERMS_VERSION,
      terms_accepted: choice.termsAccepted,
      clauses_approved: choice.clausesApproved,
    })
    return acceptance.value
  }

  return { acceptance, loaded, accepted, load, accept }
})
