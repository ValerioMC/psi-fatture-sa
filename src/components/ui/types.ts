import type { Tone } from '@/utils/labels'

/** One option of a SegmentedControl. */
export interface SegmentOption<V> {
  value: V
  label: string
  /** Optional status dot before the label. */
  tone?: Tone
  /** Optional count shown after the label, e.g. how many invoices match a filter. */
  count?: number
}

/** One option of a ComboBox. */
export interface ComboOption {
  value: number
  label: string
  /** Secondary text shown right-aligned and searched too, e.g. a codice fiscale. */
  detail?: string
}
