import { describe, expect, it } from 'vitest'
import { projectYearEnd, readThreshold } from './forfettario'

describe('projectYearEnd', () => {
  it('extends the pace so far to the whole year', () => {
    // 1 July 2026 is day 182 of 365.
    expect(projectYearEnd(40_000, 2026, new Date(2026, 6, 1))).toBe(Math.round((40_000 / 182) * 365))
  })

  it('does not project a year other than the current one', () => {
    expect(projectYearEnd(40_000, 2025, new Date(2026, 6, 1))).toBeNull()
  })

  it('waits for a month of data before projecting', () => {
    expect(projectYearEnd(3_000, 2026, new Date(2026, 0, 20))).toBeNull()
  })
})

describe('readThreshold', () => {
  it('is calm while far from the ceiling', () => {
    expect(readThreshold(30_000, 2026, new Date(2026, 8, 24)).level).toBe('ok')
  })

  it('warns when the pace leads past 85.000 €', () => {
    const reading = readThreshold(70_000, 2026, new Date(2026, 8, 24))
    expect(reading.projection).toBeGreaterThan(85_000)
    expect(reading.level).toBe('near')
  })

  it('warns at 85% of the ceiling even without a projection', () => {
    expect(readThreshold(73_000, 2025, new Date(2026, 8, 24)).level).toBe('near')
  })

  it('reports the ceiling as passed', () => {
    const reading = readThreshold(86_000, 2025, new Date(2026, 8, 24))
    expect(reading.level).toBe('over')
    expect(reading.ratio).toBeGreaterThan(1)
  })
})
