/**
 * Where a chart tooltip goes so it is never hidden under the sticky page
 * header: above its column when the room is there, otherwise beside the
 * column, held below the header's edge.
 */

export type TooltipSide = 'left' | 'right'

export type TooltipPlacement =
  | { kind: 'above' }
  | { kind: 'beside'; side: TooltipSide; top: number }

export interface TooltipGeometry {
  /** Viewport y of the column's top: the point the tooltip is anchored to. */
  anchorTop: number
  /** Viewport y of the column's container top; `top` in the result is relative to it. */
  containerTop: number
  /** Viewport y of the chart's baseline: a tooltip beside the column does not drop below it. */
  baseline: number
  /** Viewport y below which nothing covers the page. */
  visibleTop: number
  tooltipHeight: number
  /** Space between the anchor and a tooltip placed above it. */
  gap: number
  /** Breathing room kept from the visible top. */
  margin: number
  /** Early columns open to the right, late ones to the left, so the tooltip stays inside the chart. */
  preferredSide: TooltipSide
}

export function placeTooltip(geometry: TooltipGeometry): TooltipPlacement {
  const ceiling = geometry.visibleTop + geometry.margin
  if (geometry.anchorTop - geometry.gap - geometry.tooltipHeight >= ceiling) return { kind: 'above' }

  // Top-aligned with the column, kept under the header first and above the baseline second.
  const lowest = geometry.baseline - geometry.tooltipHeight
  const top = Math.max(ceiling, Math.min(geometry.anchorTop, lowest))
  return { kind: 'beside', side: geometry.preferredSide, top: top - geometry.containerTop }
}

/** Bottom edge of whatever sits over the scrolling pane: its own top, or a sticky header pinned inside it. */
export function visibleTopOf(element: HTMLElement): number {
  const scroller = element.closest<HTMLElement>('main') ?? document.documentElement
  const scrollerTop = Math.max(scroller.getBoundingClientRect().top, 0)
  const header = scroller.querySelector<HTMLElement>('[data-sticky-header]')
  if (header === null) return scrollerTop
  return Math.max(scrollerTop, header.getBoundingClientRect().bottom)
}
