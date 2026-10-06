/** Tells the user, in a toast, what a pass over the Sistema TS queue did. */
import { useToastStore } from '@/stores/toast'
import type { TsDispatchSummary } from '@/types'
import { describeDispatch } from '@/utils/sts'

export function useDispatchAnnouncement(): (summary: TsDispatchSummary) => void {
  const toast = useToastStore()
  return (summary) => {
    const line = describeDispatch(summary)
    if (line.tone === 'success') toast.notify(line.message)
    else if (line.tone === 'error') toast.notifyError(line.message)
    else toast.notifyInfo(line.message)
  }
}
