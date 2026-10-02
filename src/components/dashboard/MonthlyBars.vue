<script setup lang="ts">
/**
 * Twelve stacked columns by month of issue: paid in solid ink, still to collect
 * hatched on top, the same "not yet" hatching as the threshold meter. The
 * current month sits in a faint band, the best month carries its value in full
 * ink, and months still ahead are empty slots: no data is not zero.
 */
import { computed, ref } from 'vue'
import type { MonthlyRevenue } from '@/types'
import type { MonthPending } from '@/utils/monthlyPending'
import { formatCurrency, formatCurrencyCompact } from '@/utils/format'
import { plural } from '@/utils/labels'

const props = defineProps<{
  months: readonly MonthlyRevenue[]
  /** Index 0 is January; issued or overdue invoices not yet paid. */
  pending: readonly MonthPending[]
  /** 1-12 when the year shown is the current one, else null. */
  currentMonth: number | null
}>()

const hovered = ref<number | null>(null)

interface Column {
  month: number
  name: string
  paid: number
  paidCount: number
  pending: number
  pendingCount: number
  total: number
}

const columns = computed<Column[]>(() =>
  props.months.map((month) => {
    const pending = props.pending[month.month - 1] ?? { amount: 0, count: 0 }
    return {
      month: month.month,
      name: month.month_name,
      paid: month.revenue,
      paidCount: month.invoice_count,
      pending: pending.amount,
      pendingCount: pending.count,
      total: month.revenue + pending.amount,
    }
  }),
)

/** Rounds the axis maximum up to a step whose quarters are still clean numbers, without doubling the headroom. */
function niceCeiling(value: number): number {
  if (value <= 0) return 1000
  const magnitude = 10 ** Math.floor(Math.log10(value))
  const step = [1, 1.2, 1.6, 2, 2.4, 3, 4, 5, 6, 8, 10].find((candidate) => candidate * magnitude >= value) ?? 10
  return step * magnitude
}

const axisMax = computed(() => niceCeiling(Math.max(...columns.value.map((column) => column.total), 0)))
const ticks = computed(() => [1, 0.75, 0.5, 0.25, 0].map((share) => axisMax.value * share))

const lived = computed(() => columns.value.filter((column) => !isFuture(column.month) && column.total > 0))

/** Average invoiced over the months already lived that had any: the line the columns are read against. */
const average = computed(() => {
  if (lived.value.length < 2) return null
  return lived.value.reduce((sum, column) => sum + column.total, 0) / lived.value.length
})

const bestMonth = computed(() => lived.value.reduce<Column | null>((best, column) => (best === null || column.total > best.total ? column : best), null)?.month ?? null)

const paidTotal = computed(() => columns.value.reduce((sum, column) => sum + column.paid, 0))
const pendingTotal = computed(() => columns.value.reduce((sum, column) => sum + column.pending, 0))

function isFuture(month: number): boolean {
  return props.currentMonth !== null && month > props.currentMonth
}

function share(value: number): number {
  return (value / axisMax.value) * 100
}

/** A non-zero column never collapses below a sliver, so a small month is still visible. */
function heightOf(column: Column): string {
  if (column.total <= 0) return '0%'
  return `${Math.max(share(column.total), 1.5)}%`
}

function pendingShare(column: Column): string {
  return column.total > 0 ? `${(column.pending / column.total) * 100}%` : '0%'
}

const MONTH_ABBR = ['gen', 'feb', 'mar', 'apr', 'mag', 'giu', 'lug', 'ago', 'set', 'ott', 'nov', 'dic']
</script>

<template>
  <div class="@container/chart relative flex flex-col">
    <div class="flex min-h-52 flex-1 gap-3 pt-6">
      <!-- Y axis: clean ticks, right-aligned, recessive. -->
      <div class="tabular flex w-10 shrink-0 flex-col justify-between pb-6 text-right text-2xs text-text-subtle" aria-hidden="true">
        <span v-for="tick in ticks" :key="tick" class="-translate-y-1/2 leading-none">{{ formatCurrencyCompact(tick) }}</span>
      </div>

      <div class="relative flex-1">
        <!-- Gridlines dotted and quiet; only the baseline is solid. -->
        <div class="absolute inset-x-0 top-0 bottom-6 flex flex-col justify-between" aria-hidden="true">
          <div v-for="tick in ticks" :key="tick" :class="tick === 0 ? 'h-px bg-border-strong' : 'border-t border-dotted border-border-strong/70'" />
        </div>

        <div class="absolute inset-0 flex items-stretch">
          <div
            v-for="column in columns"
            :key="column.month"
            :data-month="column.month"
            class="relative flex flex-1 flex-col items-center"
            @mouseenter="hovered = column.month"
            @mouseleave="hovered = null"
          >
            <!-- "Now": a faint band the full height of the column, so the current month is found even when still empty. -->
            <div
              v-if="column.month === currentMonth"
              class="now-band absolute inset-x-0.5 -top-6 bottom-0 rounded-[8px]"
              aria-hidden="true"
            />
            <div class="relative flex w-full flex-1 items-end justify-center">
              <div
                v-if="hovered === column.month && column.month !== currentMonth"
                class="absolute inset-x-0.5 inset-y-0 rounded-[6px] bg-surface-hover"
                aria-hidden="true"
              />
              <div
                v-if="isFuture(column.month)"
                class="relative h-1 w-[min(30px,60%)] rounded-full bg-surface-sunken"
                aria-hidden="true"
              />
              <div
                v-else
                class="relative flex w-[min(30px,60%)] flex-col transition-[height] duration-500 ease-out-expo"
                :style="{ height: heightOf(column) }"
                aria-hidden="true"
              >
                <!-- The value sits on the column; hidden when the chart is too narrow for twelve labels. -->
                <span
                  v-if="column.total > 0"
                  class="tabular absolute bottom-full left-1/2 z-[2] mb-1 hidden rounded-[4px] bg-surface-raised px-1 py-0.5 -translate-x-1/2 whitespace-nowrap text-2xs leading-none @min-[34rem]/chart:block"
                  :class="column.month === bestMonth ? 'font-semibold text-text' : 'text-text-subtle'"
                >{{ formatCurrencyCompact(column.total) }}</span>
                <div
                  v-if="column.pending > 0"
                  class="pending shrink-0 rounded-t-[5px] text-warn"
                  :class="column.paid > 0 ? '' : 'rounded-b-[2px]'"
                  :style="{ height: pendingShare(column) }"
                />
                <div
                  v-if="column.paid > 0"
                  class="bar flex-1"
                  :class="[
                    column.pending > 0 ? '' : 'rounded-t-[5px]',
                    column.month === currentMonth ? 'bar-current' : hovered === column.month ? 'bar-hover' : '',
                  ]"
                />
              </div>
            </div>
            <span
              class="relative h-6 pt-1.5 text-2xs leading-none"
              :class="column.month === currentMonth ? 'font-semibold text-accent' : 'text-text-subtle'"
              aria-hidden="true"
            >{{ MONTH_ABBR[column.month - 1] }}</span>

            <!-- Tooltip: the month's total, then what is in and what is still out. -->
            <Transition name="popover">
              <div
                v-if="hovered === column.month && !isFuture(column.month)"
                class="pointer-events-none absolute z-(--z-overlay) min-w-44 whitespace-nowrap rounded-control border border-border bg-surface-raised px-3 py-2.5 shadow-modal"
                :style="{ bottom: `calc(${heightOf(column)} + 2.25rem)` }"
                role="presentation"
              >
                <p class="text-2xs capitalize text-text-subtle">{{ column.name }}</p>
                <p class="tabular text-base font-semibold text-text">{{ formatCurrency(column.total) }}</p>
                <dl class="mt-2 space-y-1 border-t border-border pt-2 text-2xs">
                  <div class="flex items-center gap-2">
                    <span class="swatch-paid size-2 rounded-[2px]" aria-hidden="true" />
                    <dt class="text-text-muted">Incassato</dt>
                    <dd class="tabular ml-auto pl-4 text-text">{{ formatCurrency(column.paid) }}</dd>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="pending size-2 rounded-[2px] text-warn" aria-hidden="true" />
                    <dt class="text-text-muted">Da incassare</dt>
                    <dd class="tabular ml-auto pl-4 text-text">{{ formatCurrency(column.pending) }}</dd>
                  </div>
                </dl>
                <p class="mt-1.5 text-2xs text-text-subtle">
                  {{ plural(column.paidCount + column.pendingCount, 'fattura', 'fatture') }}<template v-if="column.pendingCount > 0">, {{ column.pendingCount }} in attesa</template>
                </p>
              </div>
            </Transition>
          </div>
        </div>

        <!-- The average: a dashed rule with its value, so a month reads as above or below the usual. -->
        <div
          v-if="average !== null"
          class="pointer-events-none absolute inset-x-0 z-[1] border-t border-dashed border-accent/50"
          :style="{ bottom: `calc(1.5rem + (100% - 1.5rem) * ${average / axisMax})` }"
          aria-hidden="true"
        />
      </div>
    </div>

    <!-- Legend doubles as the year's split: what each mark means, and how much of it there is. -->
    <dl class="mt-4 flex flex-wrap items-center gap-x-6 gap-y-2 border-t border-border pt-3.5 pl-[3.25rem] text-xs">
      <div class="flex items-center gap-2">
        <span class="swatch-paid size-2.5 rounded-[3px]" aria-hidden="true" />
        <dt class="text-text-muted">Incassato</dt>
        <dd class="tabular font-semibold text-text">{{ formatCurrencyCompact(paidTotal) }}</dd>
      </div>
      <div v-if="pendingTotal > 0" class="flex items-center gap-2">
        <span class="pending size-2.5 rounded-[3px] text-warn ring-1 ring-warn/40 ring-inset" aria-hidden="true" />
        <dt class="text-text-muted">Da incassare</dt>
        <dd class="tabular font-semibold text-text">{{ formatCurrencyCompact(pendingTotal) }}</dd>
      </div>
      <div v-if="average !== null" class="flex items-center gap-2">
        <span class="w-3.5 border-t border-dashed border-accent" aria-hidden="true" />
        <dt class="text-text-muted">Media mensile</dt>
        <dd class="tabular font-semibold text-text">{{ formatCurrencyCompact(average) }}</dd>
      </div>
    </dl>

    <!-- The table view: every value, for screen readers. -->
    <table class="sr-only">
      <caption>Fatturato per mese</caption>
      <thead>
        <tr><th scope="col">Mese</th><th scope="col">Incassato</th><th scope="col">Da incassare</th></tr>
      </thead>
      <tbody>
        <tr v-for="column in columns" :key="column.month">
          <th scope="row">{{ column.name }}</th>
          <td>{{ formatCurrency(column.paid) }}</td>
          <td>{{ formatCurrency(column.pending) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
/* Paid is solid ink, lit from above: a lighter crown settling into the accent. */
.bar,
.swatch-paid {
  background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 72%, transparent), color-mix(in srgb, var(--accent) 50%, transparent));
}
.bar-hover {
  background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 88%, transparent), color-mix(in srgb, var(--accent) 66%, transparent));
}
.bar-current {
  background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 80%, white), var(--accent) 40%, var(--accent-strong));
  box-shadow: 0 6px 16px -6px color-mix(in srgb, var(--accent) 65%, transparent);
}

/* Still to collect is not money in hand: the same hue family as "waiting", hatched at 45°, never solid. */
.pending {
  background-color: color-mix(in srgb, currentColor 10%, transparent);
  background-image: repeating-linear-gradient(
    -45deg,
    color-mix(in srgb, currentColor 55%, transparent) 0 1.5px,
    transparent 1.5px 4.5px
  );
  box-shadow: inset 0 0 0 1px color-mix(in srgb, currentColor 35%, transparent);
}

.now-band {
  background: linear-gradient(180deg, transparent, color-mix(in srgb, var(--accent) 7%, transparent) 30%);
}
</style>
