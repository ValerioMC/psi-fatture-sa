<script setup lang="ts">
import { AlertTriangle } from 'lucide-vue-next'

/**
 * Simple confirmation dialog with cancel and delete actions.
 */
defineProps<{
  open: boolean
  title: string
  message: string
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" @click="emit('cancel')" />

      <!-- Dialog -->
      <div class="relative glass-card rounded-2xl shadow-2xl w-full max-w-md mx-4 p-6 animate-in">
        <div class="flex items-start gap-4">
          <div class="flex-shrink-0 w-10 h-10 rounded-full bg-red-100 flex items-center justify-center">
            <AlertTriangle class="w-5 h-5 text-red-600" />
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="text-base font-semibold text-sage-900">{{ title }}</h3>
            <p class="text-sm text-sage-600 mt-1">{{ message }}</p>
          </div>
        </div>

        <div class="flex justify-end gap-3 mt-6">
          <button
            type="button"
            class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            @click="emit('cancel')"
          >
            Annulla
          </button>
          <button
            type="button"
            class="bg-red-600 text-white hover:bg-red-700 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            @click="emit('confirm')"
          >
            Elimina
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
