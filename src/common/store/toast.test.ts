import { describe, expect, it } from 'vitest'
import { errorMessage, toastDuration, TOAST_ACTION_BONUS_MS, TOAST_DURATION_MS } from './toast'

describe('toast', () => {
  it('keeps errors longer than successes and extends for an action', () => {
    expect(TOAST_DURATION_MS.error).toBeGreaterThan(TOAST_DURATION_MS.success)
    expect(toastDuration('success', true)).toBe(TOAST_DURATION_MS.success + TOAST_ACTION_BONUS_MS)
  })

  it('translates database errors into sentences', () => {
    expect(errorMessage('error returned from database: FOREIGN KEY constraint failed')).toMatch(/collegato a fatture/)
    expect(errorMessage(new Error('Disco pieno'))).toBe('Disco pieno')
    expect(errorMessage(undefined)).toMatch(/imprevisto/)
  })
})
