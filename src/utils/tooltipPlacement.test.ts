import { describe, expect, it } from 'vitest'
import { placeTooltip, type TooltipGeometry } from './tooltipPlacement'

const geometry = (overrides: Partial<TooltipGeometry>): TooltipGeometry => ({
  anchorTop: 500,
  containerTop: 300,
  baseline: 560,
  visibleTop: 100,
  tooltipHeight: 130,
  gap: 36,
  margin: 8,
  preferredSide: 'right',
  ...overrides,
})

describe('placeTooltip', () => {
  it('keeps the tooltip above a short column when it fits under the header', () => {
    expect(placeTooltip(geometry({}))).toEqual({ kind: 'above' })
  })

  it('fits above when the room is exactly enough', () => {
    expect(placeTooltip(geometry({ anchorTop: 100 + 8 + 36 + 130 }))).toEqual({ kind: 'above' })
  })

  it('moves beside a tall column, top-aligned with it, when above would cross the header', () => {
    expect(placeTooltip(geometry({ anchorTop: 250, containerTop: 220 }))).toEqual({ kind: 'beside', side: 'right', top: 30 })
  })

  it('opens on the preferred side', () => {
    expect(placeTooltip(geometry({ anchorTop: 250, preferredSide: 'left' }))).toMatchObject({ kind: 'beside', side: 'left' })
  })

  it('holds the tooltip below the header when the column top is scrolled under it', () => {
    expect(placeTooltip(geometry({ anchorTop: 60, containerTop: 40 }))).toEqual({ kind: 'beside', side: 'right', top: 68 })
  })

  it('does not drop the tooltip below the baseline', () => {
    expect(placeTooltip(geometry({ anchorTop: 250, baseline: 300, containerTop: 150, visibleTop: 100 }))).toEqual({
      kind: 'beside',
      side: 'right',
      top: 20,
    })
  })

  it('prefers staying under the header over staying above the baseline', () => {
    expect(placeTooltip(geometry({ anchorTop: 120, baseline: 200, containerTop: 100 }))).toEqual({ kind: 'beside', side: 'right', top: 8 })
  })
})
