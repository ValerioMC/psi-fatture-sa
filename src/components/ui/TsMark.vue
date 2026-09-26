<script setup lang="ts">
/**
 * Where an invoice stands with the Sistema TS, drawn as a tessera sanitaria:
 * one small card that changes, next to the invoice seal wherever both matter.
 *
 *   none       a pencilled, dashed card: nothing transmitted
 *   queued     the card with a warm dot in its corner: waiting its turn
 *   sending    the stripe carries a travelling light, only while in flight
 *   accepted   the card is filled green with an engraved check: registered
 *   rejected   a danger-tinted card with a bar through the stripe
 *   cancelled  a grey card struck diagonally: it was there, now withdrawn
 *
 * Acceptance lands once, as a stamp, on an actual change and never on mount.
 */
import { onBeforeUnmount, ref, watch } from 'vue'
import { describeTsMark, type TsMarkKind } from '@/utils/sts'

const props = withDefaults(defineProps<{ kind: TsMarkKind; size?: number }>(), { size: 20 })

const arriving = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

watch(
  () => props.kind,
  (next, previous) => {
    if (next !== 'accepted' || previous === 'accepted') return
    arriving.value = true
    if (timer !== null) clearTimeout(timer)
    timer = setTimeout(() => { arriving.value = false }, 800)
  },
)
onBeforeUnmount(() => {
  if (timer !== null) clearTimeout(timer)
})
</script>

<template>
  <svg
    class="ts-mark shrink-0 overflow-visible"
    :class="[`mark-${kind}`, arriving ? 'is-arriving' : '']"
    :width="size"
    :height="size"
    viewBox="0 0 20 20"
    fill="none"
    role="img"
    :aria-label="describeTsMark(kind)"
  >
    <title>{{ describeTsMark(kind) }}</title>

    <g v-if="kind === 'none'">
      <rect x="2" y="4.5" width="16" height="11" rx="2.2" class="stroke-text-subtle" stroke-width="1.3" stroke-dasharray="2 1.8" />
    </g>

    <g v-else-if="kind === 'queued'">
      <rect x="2" y="4.5" width="16" height="11" rx="2.2" class="stroke-border-strong" stroke-width="1.4" />
      <path d="M2.7 8h14.6" class="stroke-border-strong" stroke-width="1.6" />
      <circle cx="16.4" cy="5.2" r="2.5" class="fill-warn stroke-surface-raised" stroke-width="1.2" />
    </g>

    <g v-else-if="kind === 'sending'">
      <rect x="2" y="4.5" width="16" height="11" rx="2.2" class="stroke-accent" stroke-width="1.4" />
      <path d="M2.7 8h14.6" class="stroke-accent-soft" stroke-width="1.8" />
      <path d="M2.7 8h4" class="mark-beam stroke-accent" stroke-width="1.8" stroke-linecap="round" />
      <rect x="4.5" y="10.8" width="3.4" height="2.4" rx="0.6" class="fill-accent" opacity="0.55" />
    </g>

    <g v-else-if="kind === 'accepted'" class="mark-stamp">
      <rect x="1.6" y="4.1" width="16.8" height="11.8" rx="2.4" class="fill-safe" />
      <rect x="3" y="5.5" width="14" height="9" rx="1.5" stroke="white" stroke-opacity="0.28" stroke-width="0.6" />
      <path d="M6.6 10.1l2.2 2.1 4.6-4.6" class="mark-check" stroke="white" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" pathLength="1" />
    </g>

    <g v-else-if="kind === 'rejected'">
      <rect x="2" y="4.5" width="16" height="11" rx="2.2" class="stroke-danger fill-danger-soft" stroke-width="1.5" />
      <path d="M2.7 8h14.6" class="stroke-danger" stroke-opacity="0.35" stroke-width="1.6" />
      <path d="M10 9.9v2.4" class="stroke-danger" stroke-width="1.7" stroke-linecap="round" />
      <circle cx="10" cy="13.9" r="0.95" class="fill-danger" />
    </g>

    <g v-else>
      <rect x="2" y="4.5" width="16" height="11" rx="2.2" class="stroke-text-subtle" stroke-width="1.3" />
      <path d="M4 16.5L16 3.5" class="stroke-text-subtle" stroke-width="1.3" stroke-linecap="round" />
    </g>
  </svg>
</template>

<style scoped>
/* In flight: a short light travels along the stripe, and only while in flight. */
.mark-beam { animation: beam 1.3s var(--ease-out-expo) infinite; }
@keyframes beam {
  0% { transform: translateX(0); opacity: 0; }
  20% { opacity: 1; }
  100% { transform: translateX(10.6px); opacity: 0; }
}

/* Accepted: the card lands like a stamp, then the check draws itself. */
.is-arriving .mark-stamp {
  transform-origin: 10px 10px;
  animation: mark-land 520ms cubic-bezier(0.2, 1.4, 0.4, 1) both;
}
.is-arriving .mark-check {
  stroke-dasharray: 1;
  animation: mark-check 340ms 220ms ease-out both;
}
@keyframes mark-land {
  0% { transform: scale(1.4) rotate(-8deg); opacity: 0; }
  55% { transform: scale(0.93) rotate(2deg); opacity: 1; }
  100% { transform: scale(1) rotate(0); }
}
@keyframes mark-check {
  from { stroke-dashoffset: 1; }
  to { stroke-dashoffset: 0; }
}
</style>
