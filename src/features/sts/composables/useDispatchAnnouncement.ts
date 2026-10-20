/** Tells the user, in a toast, what a pass over the Sistema TS queue did. */
import { useToastStore } from '@/common/store/toast'
import type { TsDispatchSummary } from '@/features/sts/model/ts'
import { describeDispatch } from '@/features/sts/lib/sts'

export function useDispatchAnnouncement(): (summary: TsDispatchSummary) => void {
  const toast = useToastStore()
  return (summary) => {
    const line = describeDispatch(summary)
    if (line.tone === 'success') toast.notify(line.message)
    else if (line.tone === 'error') toast.notifyError(line.message)
    else toast.notifyInfo(line.message)
  }
}
