<script setup lang="ts">
/**
 * Twelve columns, one series: paid amounts by month of issue. One colour for
 * every bar, the current month emphasised in full ink and the rest at a
 * lighter step, so the eye finds "now" first. Months still ahead are drawn as
 * empty slots rather than zero bars: no data is not the same as zero.
 */
import { computed, ref } from 'vue'
import type { MonthlyRevenue } from '@/types'
import { formatCurrency, formatCurrencyCompact } from '@/utils/format'
import { plural } from '@/utils/labels'

const props = defineProps<{
  months: readonly MonthlyRevenue[]
  /** 1-12 when the year shown is the current one, else null. */
  currentMonth: number | null
}>()

const hovered = ref<number | null>(null)

/** Rounds the axis maximum up to 1, 2, 2.5 or 5 × 10ⁿ so the ticks are clean numbers. */
function niceCeiling(value: number): number {
  if (value <= 0) return 1000
  const magnitude = 10 ** Math.floor(Math.log10(value))
  const step = [1, 2, 2.5, 5, 10].find((candidate) => candidate * magnitude >= value) ?? 10
  return step * magnitude
}

const axisMax = computed(() => niceCeiling(Math.max(...props.months.map((month) => month.revenue), 0)))
const ticks = computed(() => [1, 0.75, 0.5, 0.25, 0].map((share) => axisMax.value * share))

function isFuture(month: number): boolean {
  return props.currentMonth !== null && month > props.currentMonth
}

function heightOf(month: MonthlyRevenue): string {
  if (month.revenue <= 0) return '0%'
  return `${Math.max((month.revenue / axisMax.value) * 100, 1.5)}%`
}

const MONTH_ABBR = ['gen', 'feb', 'mar', 'apr', 'mag', 'giu', 'lug', 'ago', 'set', 'ott', 'nov', 'dic']
</script>

<template>
  <div class="relative">
    <div class="flex h-48 gap-3">
      <!-- Y axis: clean ticks, right-aligned, recessive. -->
      <div class="tabular flex w-10 shrink-0 flex-col justify-between pb-6 text-right text-2xs text-text-subtle" aria-hidden="true">
        <span v-for="tick in ticks" :key="tick" class="-translate-y-1/2 leading-none">{{ formatCurrencyCompact(tick) }}</span>
      </div>

      <div class="relative flex-1">
        <!-- Hairline, solid gridlines one step off the surface. -->
        <div class="absolute inset-x-0 top-0 bottom-6 flex flex-col justify-between" aria-hidden="true">
          <div v-for="tick in ticks" :key="tick" class="h-px bg-border" :class="tick === 0 ? 'bg-border-strong' : ''" />
        </div>

        <div class="absolute inset-0 flex items-stretch">
          <div
            v-for="month in months"
            :key="month.month"
            class="relative flex flex-1 flex-col items-center"
            @mouseenter="hovered = month.month"
            @mouseleave="hovered = null"
          >
            <div class="relative flex w-full flex-1 items-end justify-center">
              <!-- The hit target is the whole column, wider and taller than the bar. -->
              <div
                v-if="hovered === month.month"
                class="absolute inset-x-0.5 inset-y-0 rounded-[6px] bg-surface-hover"
                aria-hidden="true"
              />
              <div
                v-if="isFuture(month.month)"
                class="relative h-1 w-[min(22px,62%)] rounded-full bg-surface-sunken"
                aria-hidden="true"
              />
              <div
                v-else
                class="relative w-[min(22px,62%)] rounded-t-[4px] transition-[height,background-color] duration-500 ease-out-expo"
                :class="month.month === currentMonth ? 'bg-accent' : hovered === month.month ? 'bg-accent/60' : 'bg-accent/35'"
                :style="{ height: heightOf(month) }"
                aria-hidden="true"
              />
            </div>
            <span
              class="h-6 pt-1.5 text-2xs leading-none"
              :class="month.month === currentMonth ? 'font-semibold text-text' : 'text-text-subtle'"
              aria-hidden="true"
            >{{ MONTH_ABBR[month.month - 1] }}</span>

            <!-- Tooltip: value first, then how many invoices make it up. -->
            <Transition name="popover">
              <div
                v-if="hovered === month.month && !isFuture(month.month)"
                class="pointer-events-none absolute z-(--z-overlay) whitespace-nowrap rounded-control border border-border bg-surface-raised px-3 py-2 shadow-modal"
                :style="{ bottom: `calc(${heightOf(month)} + 1.25rem)` }"
                role="presentation"
              >
                <p class="text-2xs capitalize text-text-subtle">{{ month.month_name }}</p>
                <p class="tabular text-base font-semibold text-text">{{ formatCurrency(month.revenue) }}</p>
                <p class="text-2xs text-text-muted">{{ plural(month.invoice_count, 'fattura pagata', 'fatture pagate') }}</p>
              </div>
            </Transition>
          </div>
        </div>
      </div>
    </div>

    <!-- The table view: every value, for screen readers. -->
    <table class="sr-only">
      <caption>Incassato per mese</caption>
      <tbody>
        <tr v-for="month in months" :key="month.month">
          <th scope="row">{{ month.month_name }}</th>
          <td>{{ formatCurrency(month.revenue) }}</td>
          <td>{{ plural(month.invoice_count, 'fattura', 'fatture') }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
