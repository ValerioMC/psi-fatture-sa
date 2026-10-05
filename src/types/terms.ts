/** The acceptance of one version of the terms of use, with the specific approval of its limitation clauses. */
export interface TermsAcceptance {
  version: string
  clauses_approved: boolean
  accepted_at: string
}

export interface AcceptTermsInput {
  version: string
  terms_accepted: boolean
  clauses_approved: boolean
}
