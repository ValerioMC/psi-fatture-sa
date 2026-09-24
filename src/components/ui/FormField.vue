<script setup lang="ts">
/**
 * Label, control, hint and error for one form value. The slot receives the
 * ids to wire up, so the control stays a plain native element with `.field`:
 *
 *   <FormField label="Email" :error="errors.email" v-slot="{ id, invalid, describedBy }">
 *     <input :id="id" class="field" :aria-invalid="invalid" :aria-describedby="describedBy" />
 *   </FormField>
 *
 * The error replaces the hint rather than stacking under it: one line of help
 * at a time, and the layout below does not jump when validation runs.
 */
import { computed, useId } from 'vue'
import { CircleAlert } from 'lucide-vue-next'

const props = defineProps<{
  label: string
  hint?: string
  error?: string
  required?: boolean
  optional?: boolean
}>()

const id = useId()
const messageId = `${id}-message`
const invalid = computed(() => props.error !== undefined && props.error !== '')
const hasMessage = computed(() => invalid.value || props.hint !== undefined)
</script>

<template>
  <div class="min-w-0">
    <label :for="id" class="flex items-baseline gap-1.5 mb-1.5 text-sm font-medium text-text-muted">
      {{ label }}
      <span v-if="required" class="text-text-subtle" aria-hidden="true">*</span>
      <span v-if="optional" class="text-2xs font-normal text-text-subtle">facoltativo</span>
    </label>
    <slot :id="id" :invalid="invalid || undefined" :described-by="hasMessage ? messageId : undefined" />
    <p
      v-if="hasMessage"
      :id="messageId"
      class="mt-1.5 text-xs flex items-start gap-1.5"
      :class="invalid ? 'text-danger' : 'text-text-subtle'"
      :role="invalid ? 'alert' : undefined"
    >
      <CircleAlert v-if="invalid" :size="13" :stroke-width="2" class="mt-px shrink-0" aria-hidden="true" />
      <span>{{ invalid ? error : hint }}</span>
    </p>
  </div>
</template>
