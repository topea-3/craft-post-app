import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useBlocker, useLocation, useNavigate } from 'react-router-dom'
import { PostcardPreviewCanvas } from '../components/PostcardPreviewCanvas'
import { PrintLayerPanel } from '../components/PrintLayerPanel'
import {
  createPostcardSendsBatch,
  resolvePrintJobItems,
  resnapshotPrintJobItems,
} from '../api'
import { usePrintJob } from '../hooks/usePrintJob'
import { usePrintJobDraft } from '../hooks/usePrintJobDraft'
import { usePrintPostcardType } from '../hooks/usePrintPostcardType'
import {
  PRINT_COMPLETE_MESSAGE,
  PRINT_NO_VALID_ITEMS_MESSAGE,
  PRINT_OPERATION_ERROR_MESSAGE,
  PRINT_PREFS_SAVED_MESSAGE,
  PRINT_RESOLVE_INVALID_MESSAGE,
  PRINT_SEND_FAILED_MESSAGE,
  PRINT_TYPE_CHANGE_UNSAVED_MESSAGE,
  PRINT_UNSAVED_LEAVE_MESSAGE,
} from '../messages'
import type { ExcludedAlert, PostcardType, PrintJobItem } from '../types'
import { parseAddressEntriesInvalidError, POSTCARD_TYPE_OPTIONS } from '../types'

type LocationState = {
  items?: PrintJobItem[]
  excludedAlerts?: ExcludedAlert[]
}

export function PrintPreviewPage() {
  const navigate = useNavigate()
  const location = useLocation()
  const state = (location.state ?? {}) as LocationState
  const { selectedIds, removeIds, setExcludedAlerts, clearDraft } = usePrintJobDraft()
  const { postcardType, setPostcardType } = usePrintPostcardType()

  const [items, setItems] = useState<PrintJobItem[]>(state.items ?? [])
  const [pageExcludedAlerts, setPageExcludedAlerts] = useState<ExcludedAlert[]>(
    state.excludedAlerts ?? [],
  )
  const [pageIndex, setPageIndex] = useState(0)
  const [loading, setLoading] = useState(!state.items)
  const [error, setError] = useState<string | null>(null)
  const [statusMessage, setStatusMessage] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [pendingPrintJobId, setPendingPrintJobId] = useState<string | null>(null)
  const [pendingSnapshots, setPendingSnapshots] = useState<PrintJobItem[] | null>(null)
  const pendingPdfRef = useRef<{ save: (name?: string) => void } | null>(null)
  const bypassBlockerRef = useRef(false)

  const onItemsChange = useCallback((next: PrintJobItem[]) => {
    setItems(next)
  }, [])

  const job = usePrintJob({
    postcardType,
    items,
    onItemsChange,
  })

  const blocker = useBlocker(
    ({ currentLocation, nextLocation }) =>
      !bypassBlockerRef.current &&
      job.isDirty &&
      currentLocation.pathname !== nextLocation.pathname,
  )

  useEffect(() => {
    if (blocker.state !== 'blocked') return
    if (window.confirm(PRINT_UNSAVED_LEAVE_MESSAGE)) {
      blocker.proceed()
    } else {
      blocker.reset()
    }
  }, [blocker])

  useEffect(() => {
    if (!job.isDirty) return
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      e.preventDefault()
      e.returnValue = ''
    }
    window.addEventListener('beforeunload', onBeforeUnload)
    return () => window.removeEventListener('beforeunload', onBeforeUnload)
  }, [job.isDirty])

  // Load items if navigated without state (e.g. refresh → back to select)
  useEffect(() => {
    if (selectedIds.length === 0) {
      navigate('/print/select', { replace: true })
      return
    }
    if (state.items && state.items.length > 0) {
      setItems(job.ensureItemVisibility(state.items))
      return
    }
    let cancelled = false
    ;(async () => {
      setLoading(true)
      try {
        const result = await resolvePrintJobItems(selectedIds)
        if (cancelled) return
        if (result.items.length === 0) {
          setExcludedAlerts(result.excludedAlerts)
          setPageExcludedAlerts(result.excludedAlerts)
          setError(PRINT_NO_VALID_ITEMS_MESSAGE)
          navigate('/print/select')
          return
        }
        setItems(job.ensureItemVisibility(result.items))
        setExcludedAlerts(result.excludedAlerts)
        setPageExcludedAlerts(result.excludedAlerts)
      } catch (e) {
        if (cancelled) return
        const invalid = parseAddressEntriesInvalidError(e)
        if (invalid) {
          removeIds(invalid.entries.map((x) => x.address_entry_id))
          setError(PRINT_RESOLVE_INVALID_MESSAGE)
          navigate('/print/select')
          return
        }
        console.error('resolve_print_job_items failed:', e)
        setError(PRINT_OPERATION_ERROR_MESSAGE)
      } finally {
        if (!cancelled) setLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const currentItem = items[pageIndex] ?? null
  const pageLabel = useMemo(
    () => (items.length === 0 ? '0 / 0' : `${pageIndex + 1} / ${items.length}`),
    [items.length, pageIndex],
  )

  const confirmLeaveIfDirty = useCallback(() => {
    if (!job.isDirty) return true
    return window.confirm(PRINT_UNSAVED_LEAVE_MESSAGE)
  }, [job.isDirty])

  const handleBack = () => {
    if (!confirmLeaveIfDirty()) return
    bypassBlockerRef.current = true
    navigate('/print/confirm', {
      state: { items, excludedAlerts: pageExcludedAlerts },
    })
  }

  const handleCancel = () => {
    if (!confirmLeaveIfDirty()) return
    bypassBlockerRef.current = true
    clearDraft()
    navigate('/addresses')
  }

  const handleTypeChange = (next: PostcardType) => {
    if (next === postcardType) return
    if (job.isDirty && !window.confirm(PRINT_TYPE_CHANGE_UNSAVED_MESSAGE)) {
      return
    }
    setPostcardType(next)
  }

  const handleSavePrefs = async () => {
    const ok = await job.savePrefs()
    if (ok) setStatusMessage(PRINT_PREFS_SAVED_MESSAGE)
  }

  const handlePrint = async () => {
    if (busy || items.length === 0) return
    setBusy(true)
    setError(null)
    setStatusMessage(null)
    setPendingPrintJobId(null)
    setPendingSnapshots(null)
    pendingPdfRef.current = null

    let printJobId: string | null = null
    try {
      const snapped = await resnapshotPrintJobItems(items)
      const withVisibility = snapped.map((item, i) => ({
        ...item,
        layerVisibility: items[i]?.layerVisibility ?? item.layerVisibility,
      }))

      printJobId = crypto.randomUUID()
      const pdf = await (async () => {
        const { renderPrintPdf } = await import('../render/htmlCapture/renderPrintPdf')
        return renderPrintPdf({
          items: withVisibility,
          layoutSpec: job.layoutSpec,
          layoutOffsets: job.layoutOffsets,
        })
      })()

      try {
        await createPostcardSendsBatch({
          printJobId,
          postcardType,
          items: withVisibility,
        })
        printJobId = null
        pdf.save(`postcard-address-${Date.now()}.pdf`)
        setStatusMessage(PRINT_COMPLETE_MESSAGE)
      } catch (sendErr) {
        console.error('create_postcard_sends_batch failed:', sendErr)
        setPendingPrintJobId(printJobId)
        setPendingSnapshots(withVisibility)
        pendingPdfRef.current = pdf
        setError(PRINT_SEND_FAILED_MESSAGE)
      }
    } catch (e) {
      console.error('print failed:', e)
      printJobId = null
      setError(PRINT_OPERATION_ERROR_MESSAGE)
    } finally {
      setBusy(false)
    }
  }

  const handleRetrySend = async () => {
    if (busy || !pendingPrintJobId || !pendingSnapshots) return
    setBusy(true)
    setError(null)
    try {
      await createPostcardSendsBatch({
        printJobId: pendingPrintJobId,
        postcardType,
        items: pendingSnapshots,
      })
      setPendingPrintJobId(null)
      setPendingSnapshots(null)
      pendingPdfRef.current?.save(`postcard-address-${Date.now()}.pdf`)
      pendingPdfRef.current = null
      setStatusMessage(PRINT_COMPLETE_MESSAGE)
    } catch (e) {
      console.error('retry send failed:', e)
      setError(PRINT_SEND_FAILED_MESSAGE)
    } finally {
      setBusy(false)
    }
  }

  if (loading) {
    return (
      <div className="print-page">
        <p className="print-loading">読み込み中…</p>
      </div>
    )
  }

  return (
    <div className="print-page print-preview-page">
      <div className="print-page-header print-preview-header">
        <button type="button" onClick={handleBack} disabled={busy}>
          ← 戻る
        </button>
        <h1 className="print-page-title">印刷プレビュー</h1>
        <label className="print-type-select">
          <span>種別</span>
          <select
            value={postcardType}
            onChange={(e) => handleTypeChange(e.target.value as PostcardType)}
            disabled={busy || job.prefsLoading}
          >
            {POSTCARD_TYPE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          onClick={() => job.resetOffsets(job.selectedLayerId ?? undefined)}
          disabled={busy}
        >
          基準に戻す
        </button>
        <button
          type="button"
          className="print-primary-button"
          onClick={handlePrint}
          disabled={busy || items.length === 0}
        >
          {busy && !pendingPrintJobId ? '処理中…' : '印刷'}
        </button>
        {pendingPrintJobId && (
          <button type="button" onClick={handleRetrySend} disabled={busy}>
            再試行
          </button>
        )}
        <button type="button" onClick={handleCancel} disabled={busy}>
          キャンセル
        </button>
      </div>

      {items.length > 1 && (
        <div className="print-page-nav">
          <button
            type="button"
            disabled={pageIndex <= 0 || busy}
            onClick={() => setPageIndex((i) => Math.max(0, i - 1))}
          >
            ≪
          </button>
          <span>{pageLabel}</span>
          <button
            type="button"
            disabled={pageIndex >= items.length - 1 || busy}
            onClick={() => setPageIndex((i) => Math.min(items.length - 1, i + 1))}
          >
            ≫
          </button>
        </div>
      )}

      {error && <p className="print-error">{error}</p>}
      {statusMessage && <p className="print-status">{statusMessage}</p>}
      {job.prefsError && <p className="print-error">{job.prefsError}</p>}

      <div className="print-preview-body">
        {currentItem && (
          <PostcardPreviewCanvas
            item={currentItem}
            layoutSpec={job.layoutSpec}
            layoutOffsets={job.layoutOffsets}
            selectedLayerId={job.selectedLayerId}
            onSelectLayer={job.setSelectedLayerId}
            onOffsetChange={job.setOffset}
          />
        )}
        {currentItem && (
          <PrintLayerPanel
            visibility={currentItem.layerVisibility}
            selectedLayerId={job.selectedLayerId}
            onSelectLayer={job.setSelectedLayerId}
            onToggleVisibility={(id, visible) =>
              job.setLayerVisibility(pageIndex, id, visible)
            }
            onSavePrefs={handleSavePrefs}
            savingPrefs={job.savingPrefs}
            prefsDisabled={busy}
          />
        )}
      </div>
    </div>
  )
}
