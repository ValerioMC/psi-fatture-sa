<script setup lang="ts">
/**
 * Whether the invoice reached the patient's inbox, as one envelope that changes: dashed
 * when not sent, inked with a ticked seal when sent, in danger tone with a "!" seal when
 * nothing went. The seal lands once, on an actual change to sent.
 */
import { onBeforeUnmount, ref, watch } from 'vue'
import { describeMailMark, type MailMarkKind } from '@/utils/email'

const props = withDefaults(defineProps<{ kind: MailMarkKind; size?: number }>(), { size: 20 })

const arriving = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

watch(
  () => props.kind,
  (next, previous) => {
    if (next !== 'sent' || previous === 'sent') return
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
    class="mail-mark shrink-0 overflow-visible"
    :class="arriving ? 'is-arriving' : ''"
    :width="size"
    :height="size"
    viewBox="0 0 20 20"
    fill="none"
    role="img"
    :aria-label="describeMailMark(kind)"
  >
    <title>{{ describeMailMark(kind) }}</title>

    <g v-if="kind === 'none'">
      <rect x="2.2" y="4.8" width="15.6" height="11" rx="2" class="stroke-text-subtle" stroke-width="1.3" stroke-dasharray="2 1.8" />
      <path d="M3 6l7 5 7-5" class="stroke-text-subtle" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" opacity="0.7" />
    </g>

    <g v-else-if="kind === 'sent'">
      <g class="mark-envelope">
        <rect x="2.2" y="4.8" width="15.6" height="11" rx="2" class="fill-accent-soft stroke-accent" stroke-width="1.4" />
        <path d="M3 6l7 5 7-5" class="stroke-accent" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </g>
      <g class="mark-seal">
        <circle cx="16.2" cy="15.2" r="3.6" class="fill-accent stroke-surface-raised" stroke-width="1.3" />
        <path d="M14.6 15.3l1.1 1.1 2.1-2.2" stroke="var(--accent-ink)" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
      </g>
    </g>

    <g v-else>
      <rect x="2.2" y="4.8" width="15.6" height="11" rx="2" class="fill-danger-soft stroke-danger" stroke-width="1.4" />
      <path d="M3 6l7 5 7-5" class="stroke-danger" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" stroke-opacity="0.6" />
      <circle cx="16.2" cy="15.2" r="3.6" class="fill-danger stroke-surface-raised" stroke-width="1.3" />
      <path d="M16.2 13.6v1.9" stroke="white" stroke-width="1.3" stroke-linecap="round" />
      <circle cx="16.2" cy="17" r="0.6" fill="white" />
    </g>
  </svg>
</template>

<style scoped>
/* Sent: the envelope settles and the seal is pressed on its corner. */
.is-arriving .mark-envelope {
  transform-origin: 10px 10px;
  animation: envelope-settle 420ms var(--ease-out-expo) both;
}
.is-arriving .mark-seal {
  transform-origin: 16.2px 15.2px;
  animation: seal-press 520ms 120ms cubic-bezier(0.2, 1.4, 0.4, 1) both;
}
@keyframes envelope-settle {
  from { transform: translate(-3px, 1px); opacity: 0.4; }
  to { transform: none; opacity: 1; }
}
@keyframes seal-press {
  0% { transform: scale(1.6); opacity: 0; }
  60% { transform: scale(0.9); opacity: 1; }
  100% { transform: scale(1); }
}
</style>
