<script setup lang="ts">
/** The terms of use as numbered clauses, the same text on the acceptance screen and in Settings. */
import { TERMS_CLAUSES } from '@/legal/terms'
</script>

<template>
  <ol class="space-y-5">
    <li v-for="clause in TERMS_CLAUSES" :key="clause.number">
      <h3 class="text-base font-semibold text-text">{{ clause.number }}. {{ clause.title }}</h3>
      <p v-for="paragraph in clause.paragraphs" :key="paragraph" class="mt-1.5 text-sm leading-relaxed text-text-muted">{{ paragraph }}</p>
    </li>
  </ol>
</template>
