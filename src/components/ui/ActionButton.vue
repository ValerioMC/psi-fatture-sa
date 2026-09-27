<script setup lang="ts">
/**
 * A button that shows its outcome where the eye already is. `run` answers true, false or
 * "cancelled". Launch: the label fades and the icon leaves ("send" flies the plane out).
 * Waiting: three dots. Done: a ring and tick draw in the button's own colour, then `done`.
 */
import { computed, onBeforeUnmount, ref, useAttrs, useSlots, type Component } from 'vue'
import { Send } from 'lucide-vue-next'
import { BUTTON_BASE, BUTTON_SIZE, BUTTON_VARIANT, type ButtonSize } from './buttonStyles'

type Phase = 'idle' | 'launch' | 'waiting' | 'done' | 'failed' | 'return'

const LAUNCH_MS = 560
const HOLD_MS = 1300
const SETTLE_MS = 380

defineOptions({ inheritAttrs: false })

const props = withDefaults(
  defineProps<{
    run: () => Promise<boolean | 'cancelled'>
    motion?: 'send' | 'confirm'
    variant?: 'primary' | 'secondary' | 'ghost'
    size?: ButtonSize
    /** The resting icon; "send" defaults to the paper plane. */
    icon?: Component
    doneLabel?: string
    /** What a screen reader hears while `run` is out. */
    workingLabel?: string
    block?: boolean
    disabled?: boolean
  }>(),
  { motion: 'confirm', variant: 'primary', size: 'md', doneLabel: 'Fatto', workingLabel: 'In corso', block: false, disabled: false },
)

const emit = defineEmits<{ done: [] }>()
const attrs = useAttrs()
const slots = useSlots()

const phase = ref<Phase>('idle')
const buttonRef = ref<HTMLButtonElement | null>(null)
const flight = ref('8rem')
let alive = true
onBeforeUnmount(() => {
  alive = false
})

const glyph = computed(() => props.icon ?? (props.motion === 'send' ? Send : undefined))
const busy = computed(() => phase.value !== 'idle')
const iconSize = computed(() => BUTTON_SIZE[props.size].iconSize)
const announcement = computed(() => {
  if (phase.value === 'launch' || phase.value === 'waiting') return props.workingLabel
  if (phase.value === 'done') return props.doneLabel
  return ''
})

const classes = computed(() => [
  BUTTON_BASE,
  'overflow-hidden',
  BUTTON_VARIANT[props.variant],
  slots.default ? BUTTON_SIZE[props.size].text : BUTTON_SIZE[props.size].icon,
  props.block ? 'w-full' : '',
  // Busy is not disabled: the button keeps its colour while it shows the outcome, it just ignores clicks.
  busy.value ? 'cursor-default active:translate-y-0' : '',
])

function pause(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function settle(next: Phase): Promise<void> {
  phase.value = next
  await pause(SETTLE_MS)
  if (alive) phase.value = 'idle'
}

/** Runs the action with its animation; also called by a parent form on submit. */
async function trigger(): Promise<void> {
  if (busy.value || props.disabled) return
  flight.value = `${(buttonRef.value?.offsetWidth ?? 128) + 8}px`
  phase.value = 'launch'
  let failure: unknown = null
  const outcome = props.run().catch((error: unknown): false => {
    failure = error
    return false
  })
  const launched = pause(LAUNCH_MS)
  const first = await Promise.race([outcome.then(() => 'answered' as const), launched.then(() => 'launched' as const)])
  if (first === 'launched' && alive) phase.value = 'waiting'
  const [ok] = await Promise.all([outcome, launched])
  if (!alive) return
  if (ok === 'cancelled') {
    await settle('return')
    return
  }
  if (!ok) {
    await settle('failed')
    if (failure !== null) throw failure
    return
  }
  phase.value = 'done'
  await pause(HOLD_MS)
  if (!alive) return
  emit('done')
  await settle('return')
}

defineExpose({ trigger })
</script>

<template>
  <button
    ref="buttonRef"
    v-bind="attrs"
    type="button"
    :class="classes"
    :style="{ '--flight': flight }"
    :data-phase="phase"
    :data-motion="motion"
    :disabled="disabled"
    :aria-busy="phase === 'launch' || phase === 'waiting' || undefined"
    @click="trigger"
  >
    <span class="action-rest inline-flex items-center" :class="size === 'sm' ? 'gap-1.5' : 'gap-2'">
      <span v-if="glyph" class="action-glyph inline-flex">
        <component :is="glyph" :size="iconSize" :stroke-width="1.75" aria-hidden="true" />
      </span>
      <span v-if="$slots.default" class="action-label"><slot /></span>
    </span>

    <span v-if="phase === 'waiting'" class="absolute inset-0 flex items-center justify-center gap-1" aria-hidden="true">
      <span class="action-dot" /><span class="action-dot" /><span class="action-dot" />
    </span>

    <span v-if="phase === 'done'" class="absolute inset-0 flex items-center justify-center gap-1.5" aria-hidden="true">
      <svg class="action-tick shrink-0" :width="iconSize + 2" :height="iconSize + 2" viewBox="0 0 20 20" fill="none">
        <circle class="tick-ring" cx="10" cy="10" r="8.25" stroke="currentColor" stroke-width="1.6" pathLength="1" />
        <path class="tick-mark" d="M6.3 10.3l2.5 2.5 4.9-5.1" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" pathLength="1" />
      </svg>
      <span v-if="$slots.default" class="action-done-label">{{ doneLabel }}</span>
    </span>
  </button>
  <span class="sr-only" role="status" aria-live="polite">{{ announcement }}</span>
</template>

<style scoped>
.action-glyph,
.action-label {
  transition: opacity 160ms ease, transform 220ms var(--ease-out-expo);
}

/* While the action is out, the label steps back so the motion reads on its own. */
[data-phase='launch'] .action-label,
[data-phase='waiting'] .action-label,
[data-phase='done'] .action-label {
  opacity: 0;
  transform: translateX(-3px);
}

/* Send: a small wind-up, then the plane crosses the button and leaves by its right edge. */
[data-motion='send'][data-phase='launch'] .action-glyph,
[data-motion='send'][data-phase='waiting'] .action-glyph,
[data-motion='send'][data-phase='done'] .action-glyph {
  animation: plane-away 560ms cubic-bezier(0.55, 0, 0.7, 0.25) forwards;
}
@keyframes plane-away {
  0% { transform: none; opacity: 1; }
  18% { transform: translate(-3px, 1.5px) rotate(-10deg); opacity: 1; }
  82% { opacity: 1; }
  100% { transform: translate(var(--flight), -7px) rotate(6deg); opacity: 0; }
}

/* Confirm: the icon shrinks away to leave the stage to the tick. */
[data-motion='confirm'][data-phase='launch'] .action-glyph,
[data-motion='confirm'][data-phase='waiting'] .action-glyph,
[data-motion='confirm'][data-phase='done'] .action-glyph {
  opacity: 0;
  transform: scale(0.6);
}

/* Back to rest: the icon comes in from the left, as if the next one were ready. */
[data-phase='return'] .action-glyph,
[data-phase='failed'] .action-glyph {
  animation: glyph-return 380ms var(--ease-out-expo) both;
}
@keyframes glyph-return {
  from { transform: translateX(-10px); opacity: 0; }
  to { transform: none; opacity: 1; }
}

[data-phase='failed'] .action-rest { animation: nudge 360ms ease-in-out; }
@keyframes nudge {
  0%, 100% { transform: none; }
  25% { transform: translateX(-3px); }
  50% { transform: translateX(3px); }
  75% { transform: translateX(-1.5px); }
}

/* Waiting: three dots breathe in turn, and exist only while waiting. */
.action-dot {
  width: 4px;
  height: 4px;
  border-radius: 99px;
  background: currentColor;
  animation: breathe 900ms ease-in-out infinite;
}
.action-dot:nth-child(2) { animation-delay: 150ms; }
.action-dot:nth-child(3) { animation-delay: 300ms; }
@keyframes breathe {
  0%, 100% { opacity: 0.25; transform: scale(0.8); }
  50% { opacity: 1; transform: scale(1); }
}

/* Done: the ring draws, the tick follows, the word settles in. Same colour as the label. */
.tick-ring {
  stroke-dasharray: 1;
  stroke-dashoffset: 1;
  transform-origin: 10px 10px;
  animation: draw 420ms ease-out forwards, ring-pop 520ms var(--ease-out-expo);
}
.tick-mark {
  stroke-dasharray: 1;
  stroke-dashoffset: 1;
  animation: draw 260ms 320ms ease-out forwards;
}
.action-done-label { animation: done-label 260ms 240ms ease-out both; }
@keyframes draw { to { stroke-dashoffset: 0; } }
@keyframes ring-pop {
  0% { transform: scale(0.7) rotate(-90deg); }
  100% { transform: scale(1) rotate(-90deg); }
}
.tick-ring { transform: rotate(-90deg); }
@keyframes done-label {
  from { opacity: 0; transform: translateX(-3px); }
  to { opacity: 1; transform: none; }
}
</style>
