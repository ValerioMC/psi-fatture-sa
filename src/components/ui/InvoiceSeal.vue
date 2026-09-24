<script setup lang="ts">
/**
 * The invoice seal: one small drawn object that tells an invoice's life at a
 * glance, used wherever an invoice appears.
 *
 * It is one shape that changes, not five icons, so the reader learns "the
 * ring closing means paid" instead of matching unrelated glyphs.
 *
 *   draft      A dashed ring, pencilled in: not yet a document.
 *   issued     A hairline ring with a warm arc running clockwise from twelve
 *              o'clock. The arc is the payment window: it is empty on the
 *              day of issue and closes on the due date. No due date → a quiet
 *              solid ring with a centre point.
 *   overdue    The arc has closed and turned danger-red; a short mark in the
 *              centre. In detail views (`live`) the ring breathes slowly, and
 *              only while the invoice stays overdue.
 *   paid       The ring is struck: a filled green disc with an engraved check
 *              and a faint inner ring, like a stamp pressed into paper.
 *   cancelled  A grey ring with a single diagonal stroke through it.
 *
 * Arrivals play once, on an actual change of state and never on mount: the
 * stamp lands with a small overshoot when an invoice becomes paid; the ring
 * draws itself when a draft is issued.
 */
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import type { InvoiceStatus } from '@/types'
import { deriveSeal, describeSeal, type SealKind } from '@/utils/invoiceSeal'

const props = withDefaults(
  defineProps<{
    status: InvoiceStatus
    issueDate: string
    dueDate?: string | null
    size?: number
    live?: boolean
  }>(),
  { size: 20, live: false, dueDate: null },
)

const RADIUS = 8
const CIRCUMFERENCE = 2 * Math.PI * RADIUS

const seal = computed(() => deriveSeal({ status: props.status, issue_date: props.issueDate, due_date: props.dueDate }))
const description = computed(() => describeSeal(seal.value))

/** Arc length of the payment window already used, for the issued clock. */
const arcDash = computed(() => {
  const elapsed = seal.value.elapsed ?? 0
  const length = Math.max(0.0001, CIRCUMFERENCE * elapsed)
  return `${length} ${CIRCUMFERENCE}`
})

const arrival = ref<'stamp' | 'draw' | null>(null)
let arrivalTimer: ReturnType<typeof setTimeout> | null = null

watch(
  () => seal.value.kind,
  (next: SealKind, previous: SealKind) => {
    if (next === previous) return
    if (next === 'paid') arrival.value = 'stamp'
    else if (next === 'issued' && previous === 'draft') arrival.value = 'draw'
    else return
    if (arrivalTimer !== null) clearTimeout(arrivalTimer)
    arrivalTimer = setTimeout(() => { arrival.value = null }, 900)
  },
)
onBeforeUnmount(() => {
  if (arrivalTimer !== null) clearTimeout(arrivalTimer)
})
</script>

<template>
  <svg
    class="invoice-seal shrink-0 overflow-visible"
    :class="[`seal-${seal.kind}`, arrival ? `arrive-${arrival}` : '', live && seal.kind === 'overdue' ? 'is-live' : '']"
    :width="size"
    :height="size"
    viewBox="0 0 20 20"
    fill="none"
    role="img"
    :aria-label="description"
  >
    <title>{{ description }}</title>

    <!-- draft: pencilled dashed ring -->
    <g v-if="seal.kind === 'draft'">
      <circle cx="10" cy="10" :r="RADIUS" class="stroke-text-subtle" stroke-width="1.4" stroke-dasharray="2.1 2.1" stroke-linecap="round" />
    </g>

    <!-- issued: the payment-window clock -->
    <g v-else-if="seal.kind === 'issued'" class="seal-ring">
      <circle cx="10" cy="10" :r="RADIUS" class="stroke-border-strong" stroke-width="1.5" />
      <circle
        v-if="seal.elapsed !== null"
        cx="10"
        cy="10"
        :r="RADIUS"
        class="stroke-warn seal-arc"
        stroke-width="1.9"
        stroke-linecap="round"
        :stroke-dasharray="arcDash"
        transform="rotate(-90 10 10)"
      />
      <circle cx="10" cy="10" r="1.6" class="fill-warn" />
    </g>

    <!-- overdue: the clock has run out -->
    <g v-else-if="seal.kind === 'overdue'" class="seal-overdue">
      <circle cx="10" cy="10" :r="RADIUS" class="stroke-danger fill-danger-soft" stroke-width="1.9" />
      <path d="M10 6.2v4.6" class="stroke-danger" stroke-width="1.9" stroke-linecap="round" />
      <circle cx="10" cy="13.6" r="1.05" class="fill-danger" />
    </g>

    <!-- paid: the stamp, pressed -->
    <g v-else-if="seal.kind === 'paid'" class="seal-stamp">
      <circle cx="10" cy="10" r="9" class="fill-safe" />
      <circle cx="10" cy="10" r="7.1" stroke="white" stroke-opacity="0.28" stroke-width="0.7" />
      <path d="M6.4 10.3l2.5 2.4 4.9-5.2" class="seal-check" stroke="white" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" pathLength="1" />
    </g>

    <!-- cancelled: struck through -->
    <g v-else>
      <circle cx="10" cy="10" :r="RADIUS" class="stroke-text-subtle" stroke-width="1.4" />
      <path d="M5 15L15 5" class="stroke-text-subtle" stroke-width="1.4" stroke-linecap="round" />
    </g>
  </svg>
</template>

<style scoped>
.seal-arc { transition: stroke-dasharray 600ms var(--ease-out-expo); }

/* Stamp: lands from slightly above and larger, overshoots, settles. */
.arrive-stamp .seal-stamp {
  transform-origin: 10px 10px;
  animation: stamp-land 520ms cubic-bezier(0.2, 1.4, 0.4, 1) both;
}
.arrive-stamp .seal-check {
  stroke-dasharray: 1;
  animation: check-draw 360ms 220ms ease-out both;
}
@keyframes stamp-land {
  0% { transform: scale(1.45) rotate(-10deg); opacity: 0; }
  55% { transform: scale(0.92) rotate(2deg); opacity: 1; }
  100% { transform: scale(1) rotate(0); }
}
@keyframes check-draw {
  from { stroke-dashoffset: 1; }
  to { stroke-dashoffset: 0; }
}

/* Issue: the ring draws itself once. */
.arrive-draw .seal-ring circle:first-child {
  stroke-dasharray: 51;
  animation: ring-draw 700ms var(--ease-out-expo) both;
}
@keyframes ring-draw {
  from { stroke-dashoffset: 51; }
  to { stroke-dashoffset: 0; }
}

/* Overdue, in a detail view only: a slow breath while the state holds. */
.is-live .seal-overdue {
  transform-origin: 10px 10px;
  animation: breathe 2.8s ease-in-out infinite;
}
@keyframes breathe {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.62; }
}
</style>
