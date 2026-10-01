import { describe, expect, it } from 'vitest'
import { SPECIFICALLY_APPROVED, TERMS_CLAUSES, TERMS_VERSION, specificApprovalText } from './terms'

describe('terms of use', () => {
  it('numbers the clauses from 1 without gaps', () => {
    expect(TERMS_CLAUSES.map((clause) => clause.number)).toEqual(TERMS_CLAUSES.map((_, index) => index + 1))
  })

  it('approves specifically only clauses that exist, the liability one included', () => {
    const numbers = TERMS_CLAUSES.map((clause) => clause.number)
    for (const number of SPECIFICALLY_APPROVED) expect(numbers).toContain(number)
    const liability = TERMS_CLAUSES.find((clause) => clause.title === 'Limitazione di responsabilità')
    expect(SPECIFICALLY_APPROVED).toContain(liability?.number)
  })

  it('names every specifically approved clause by number and title', () => {
    expect(specificApprovalText()).toBe(
      'Ai sensi degli artt. 1341 e 1342 del Codice civile approvo specificamente i punti 2 (Fornitura senza garanzia), ' +
        '3 (Importi e contenuto delle fatture), 4 (Sistema TS ed email), 5 (Dati e backup) e 6 (Limitazione di responsabilità).',
    )
  })

  it('dates the version as YYYY-MM-DD', () => {
    expect(TERMS_VERSION).toMatch(/^\d{4}-\d{2}-\d{2}$/)
  })
})
