/** The look shared by AppButton and ActionButton: one closed set of variants and sizes. */
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'danger-quiet'
export type ButtonSize = 'sm' | 'md'

export const BUTTON_BASE =
  'relative inline-flex items-center justify-center rounded-control font-medium whitespace-nowrap select-none ' +
  'transition-[background-color,border-color,color,box-shadow,filter] duration-150 active:translate-y-px ' +
  'focus-ring disabled:opacity-50 aria-disabled:opacity-50 aria-disabled:pointer-events-none'

export const BUTTON_VARIANT: Readonly<Record<ButtonVariant, string>> = {
  primary:
    'bg-accent text-accent-ink hover:bg-accent-strong shadow-[inset_0_1px_0_rgb(255_255_255/0.16),0_1px_2px_rgb(20_18_60/0.18)]',
  secondary:
    'bg-surface-raised text-text border border-border-strong hover:bg-surface-hover shadow-[0_1px_1px_rgb(40_34_20/0.04)]',
  ghost: 'text-text-muted hover:text-text hover:bg-surface-hover',
  danger: 'bg-danger text-white hover:brightness-110',
  'danger-quiet': 'text-text-subtle hover:text-danger hover:bg-danger-soft',
}

export const BUTTON_SIZE: Readonly<Record<ButtonSize, { text: string; icon: string; iconSize: number }>> = {
  md: { text: 'h-control px-3.5 text-base gap-2', icon: 'size-control', iconSize: 16 },
  sm: { text: 'h-control-sm px-2.5 text-sm gap-1.5', icon: 'size-control-sm', iconSize: 15 },
}
