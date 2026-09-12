import { useCallback, useEffect, useState } from 'react'
import type { ExcludedAlert, PrintJobDraft } from '../types'
import { MAX_PRINT_SELECTION } from '../types'

const STORAGE_KEY = 'printJobDraft'

function dedupeIdsPreserveOrder(ids: string[]): string[] {
  const seen = new Set<string>()
  const out: string[] = []
  for (const id of ids) {
    if (seen.has(id)) continue
    seen.add(id)
    out.push(id)
  }
  return out
}

function readDraft(): PrintJobDraft {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY)
    if (!raw) return { addressEntryIds: [] }
    const parsed = JSON.parse(raw) as PrintJobDraft
    if (!parsed || !Array.isArray(parsed.addressEntryIds)) {
      return { addressEntryIds: [] }
    }
    return {
      addressEntryIds: dedupeIdsPreserveOrder(parsed.addressEntryIds).slice(
        0,
        MAX_PRINT_SELECTION,
      ),
      excludedAlerts: parsed.excludedAlerts,
    }
  } catch {
    return { addressEntryIds: [] }
  }
}

function writeDraft(draft: PrintJobDraft): void {
  sessionStorage.setItem(STORAGE_KEY, JSON.stringify(draft))
}

export function clearPrintJobDraft(): void {
  sessionStorage.removeItem(STORAGE_KEY)
}

export function usePrintJobDraft() {
  const [draft, setDraftState] = useState<PrintJobDraft>(() => readDraft())

  useEffect(() => {
    writeDraft(draft)
  }, [draft])

  const setDraft = useCallback((updater: PrintJobDraft | ((prev: PrintJobDraft) => PrintJobDraft)) => {
    setDraftState((prev) => {
      const next = typeof updater === 'function' ? updater(prev) : updater
      return {
        addressEntryIds: dedupeIdsPreserveOrder(next.addressEntryIds).slice(
          0,
          MAX_PRINT_SELECTION,
        ),
        excludedAlerts: next.excludedAlerts,
      }
    })
  }, [])

  const setSelectedIds = useCallback(
    (ids: string[]) => {
      setDraft((prev) => ({
        ...prev,
        addressEntryIds: ids.slice(0, MAX_PRINT_SELECTION),
      }))
    },
    [setDraft],
  )

  const toggleId = useCallback(
    (id: string) => {
      setDraft((prev) => {
        const has = prev.addressEntryIds.includes(id)
        if (has) {
          return {
            ...prev,
            addressEntryIds: prev.addressEntryIds.filter((x) => x !== id),
          }
        }
        if (prev.addressEntryIds.length >= MAX_PRINT_SELECTION) {
          return prev
        }
        return {
          ...prev,
          addressEntryIds: [...prev.addressEntryIds, id],
        }
      })
    },
    [setDraft],
  )

  const removeIds = useCallback(
    (ids: string[]) => {
      const removeSet = new Set(ids)
      setDraft((prev) => ({
        ...prev,
        addressEntryIds: prev.addressEntryIds.filter((id) => !removeSet.has(id)),
      }))
    },
    [setDraft],
  )

  /**
   * Apply set-difference prune: remove only IDs that were in `requestedIds`
   * and are absent from `activeIds`. IDs added after the request keep.
   */
  const applyActiveFilterDiff = useCallback(
    (requestedIds: string[], activeIds: string[]): number => {
      const activeSet = new Set(activeIds)
      const requestSet = new Set(requestedIds)
      let removed = 0
      setDraft((prev) => {
        const nextIds = prev.addressEntryIds.filter((id) => {
          if (requestSet.has(id) && !activeSet.has(id)) {
            return false
          }
          return true
        })
        removed = prev.addressEntryIds.length - nextIds.length
        return { ...prev, addressEntryIds: nextIds }
      })
      // Count against the request set for the message (requested ∩ missing)
      removed = requestedIds.filter((id) => !activeSet.has(id)).length
      return removed
    },
    [setDraft],
  )

  const setExcludedAlerts = useCallback(
    (alerts: ExcludedAlert[]) => {
      setDraft((prev) => ({ ...prev, excludedAlerts: alerts }))
    },
    [setDraft],
  )

  const clearDraft = useCallback(() => {
    clearPrintJobDraft()
    setDraftState({ addressEntryIds: [] })
  }, [])

  return {
    draft,
    selectedIds: draft.addressEntryIds,
    excludedAlerts: draft.excludedAlerts ?? [],
    setDraft,
    setSelectedIds,
    toggleId,
    removeIds,
    applyActiveFilterDiff,
    setExcludedAlerts,
    clearDraft,
  }
}
