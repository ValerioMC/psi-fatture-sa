/** The invoice list's year, status and search. Year and status live in the URL, so a link can open the list filtered. */
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { listQuery, statusFromQuery, yearFromQuery, type StatusFilter } from '@/utils/invoiceList'

export function useInvoiceListFilters(currentYear: number) {
  const route = useRoute()
  const router = useRouter()
  /** 0 means every year. */
  const year = ref<number>(yearFromQuery(route.query.year, currentYear))
  const status = ref<StatusFilter>(statusFromQuery(route.query.status))
  const search = ref(String(route.query.q ?? ''))

  watch([year, status], () => {
    void router.replace({ query: listQuery(year.value, status.value, currentYear) })
  })

  function clearFilters(): void {
    search.value = ''
    status.value = 'all'
  }

  return { year, status, search, clearFilters }
}
