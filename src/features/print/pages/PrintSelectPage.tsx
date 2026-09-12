import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Link, useNavigate } from 'react-router-dom'
import { PaginationControls } from '../../../components/PaginationControls'
import {
  formatAddressSingleLine,
  formatDisplayName,
  formatPostalCode,
} from '../../address/types'
import { useAddressEntryList } from '../../address/useAddressEntryList'
import { filterActiveAddressEntryIds, resolvePrintJobItems } from '../api'
import { usePrintJobDraft } from '../hooks/usePrintJobDraft'
import {
  PRINT_EXCLUDED_BANNER,
  PRINT_NO_VALID_ITEMS_MESSAGE,
  PRINT_OPERATION_ERROR_MESSAGE,
  PRINT_PRUNE_MESSAGE,
  PRINT_RESOLVE_INVALID_MESSAGE,
  PRINT_SELECT_EMPTY_MESSAGE,
  PRINT_SELECT_LABELS_PENDING_MESSAGE,
  PRINT_SELECT_MAX_MESSAGE,
  PRINT_SELECT_NO_OK_ON_PAGE_MESSAGE,
} from '../messages'
import {
  excludedAlertLabel,
  excludedReasonLabel,
  MAX_PRINT_SELECTION,
  parseAddressEntriesInvalidError,
} from '../types'

const PAGE_SIZE = 20

export function PrintSelectPage() {
  const navigate = useNavigate()
  const {
    selectedIds,
    excludedAlerts,
    toggleId,
    removeIds,
    applyActiveFilterDiff,
    setExcludedAlerts,
    setSelectedIds,
    clearDraft,
  } = usePrintJobDraft()

  const [searchText, setSearchText] = useState('')
  const [page, setPage] = useState(1)
  const [pruneMessage, setPruneMessage] = useState<string | null>(null)
  const [bannerError, setBannerError] = useState<string | null>(null)
  const [senderLabels, setSenderLabels] = useState<Record<string, string | null>>({})
  const [resolving, setResolving] = useState(false)
  const pruneDoneRef = useRef(false)

  const { items, total, isLoading, error } = useAddressEntryList({
    searchText,
    sortKey: 'nameKana',
    sortOrder: 'asc',
    page,
    pageSize: PAGE_SIZE,
  })

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))
  const currentPage = Math.min(page, totalPages)
  const selectedCount = selectedIds.length

  // Entrance prune: filter_active on draft IDs (set-difference only)
  useEffect(() => {
    if (pruneDoneRef.current) return
    pruneDoneRef.current = true
    const requestedIds = [...selectedIds]
    if (requestedIds.length === 0) return

    let cancelled = false
    ;(async () => {
      try {
        const active = await filterActiveAddressEntryIds(requestedIds)
        if (cancelled) return
        const removed = applyActiveFilterDiff(requestedIds, active)
        if (removed > 0) {
          setPruneMessage(PRINT_PRUNE_MESSAGE(removed))
        }
      } catch (e) {
        // IPC/DB failure → do not touch draft
        console.error('filter_active_address_entry_ids failed:', e)
      }
    })()
    return () => {
      cancelled = true
    }
    // intentionally once on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Resolve sender link labels for visible rows
  useEffect(() => {
    let cancelled = false
    ;(async () => {
      const next: Record<string, string | null> = {}
      await Promise.all(
        items.map(async (item) => {
          try {
            const senderId = await invoke<string | null>('get_sender_id_by_address_entry_id', {
              addressEntryId: item.id,
            })
            if (!senderId) {
              next[item.id] = null
              return
            }
            const sender = await invoke<{ label: string; archived: boolean }>('get_sender_entry', {
              id: senderId,
            })
            next[item.id] = sender.archived ? null : sender.label
          } catch {
            next[item.id] = null
          }
        }),
      )
      if (!cancelled) {
        setSenderLabels((prev) => ({ ...prev, ...next }))
      }
    })()
    return () => {
      cancelled = true
    }
  }, [items])

  const handleToggle = (id: string, excluded: boolean, known: boolean) => {
    const checked = selectedIds.includes(id)
    // ラベル未解決・除外行でも既選択なら解除を許可（ラベル遅延到着でロックされるのを防ぐ）
    if (!known && !checked) return
    if (excluded && !checked) return
    if (!checked && selectedIds.length >= MAX_PRINT_SELECTION) {
      setBannerError(PRINT_SELECT_MAX_MESSAGE)
      return
    }
    setBannerError(null)
    toggleId(id)
  }

  const labelsReady =
    items.length === 0 ||
    items.every((item) => Object.prototype.hasOwnProperty.call(senderLabels, item.id))

  const pageOkIds = items
    .filter((item) => {
      const known = Object.prototype.hasOwnProperty.call(senderLabels, item.id)
      const senderLabel = senderLabels[item.id]
      return known && senderLabel !== null
    })
    .map((item) => item.id)

  const pageIdSet = new Set(items.map((item) => item.id))
  const hasSelectionOnPage = selectedIds.some((id) => pageIdSet.has(id))
  const bulkDisabled = resolving || isLoading

  const handleSelectAllOk = () => {
    if (bulkDisabled) return
    if (!labelsReady) {
      setBannerError(PRINT_SELECT_LABELS_PENDING_MESSAGE)
      return
    }
    if (pageOkIds.length === 0) {
      setBannerError(PRINT_SELECT_NO_OK_ON_PAGE_MESSAGE)
      return
    }

    const notYetSelected = pageOkIds.filter((id) => !selectedIds.includes(id))
    if (notYetSelected.length === 0) {
      setBannerError(null)
      return
    }
    const room = MAX_PRINT_SELECTION - selectedIds.length
    if (room <= 0) {
      setBannerError(PRINT_SELECT_MAX_MESSAGE)
      return
    }

    const toAdd = notYetSelected.slice(0, room)
    setSelectedIds((prev) => {
      const seen = new Set(prev)
      const next = [...prev]
      for (const id of toAdd) {
        if (seen.has(id)) continue
        if (next.length >= MAX_PRINT_SELECTION) break
        next.push(id)
        seen.add(id)
      }
      return next
    })
    setBannerError(notYetSelected.length > room ? PRINT_SELECT_MAX_MESSAGE : null)
  }

  const handleDeselectPage = () => {
    if (bulkDisabled) return
    setBannerError(null)
    setSelectedIds((prev) => prev.filter((id) => !pageIdSet.has(id)))
  }

  const handleCancel = () => {
    clearDraft()
    navigate('/addresses')
  }

  const handleProceed = async () => {
    if (selectedIds.length === 0) {
      setBannerError(PRINT_SELECT_EMPTY_MESSAGE)
      return
    }
    setResolving(true)
    setBannerError(null)
    try {
      const result = await resolvePrintJobItems(selectedIds)
      if (result.items.length === 0) {
        setExcludedAlerts(result.excludedAlerts)
        setBannerError(PRINT_NO_VALID_ITEMS_MESSAGE)
        return
      }
      setExcludedAlerts(result.excludedAlerts)
      navigate('/print/confirm', { state: { items: result.items, excludedAlerts: result.excludedAlerts } })
    } catch (e) {
      const invalid = parseAddressEntriesInvalidError(e)
      if (invalid) {
        removeIds(invalid.entries.map((x) => x.address_entry_id))
        setBannerError(PRINT_RESOLVE_INVALID_MESSAGE)
        return
      }
      console.error('resolve_print_job_items failed:', e)
      setBannerError(PRINT_OPERATION_ERROR_MESSAGE)
    } finally {
      setResolving(false)
    }
  }

  return (
    <div className="print-page">
      <div className="print-page-header">
        <h1 className="print-page-title">印刷対象の選択</h1>
        <p className="print-page-meta">
          選択: {selectedCount} / {MAX_PRINT_SELECTION}
        </p>
      </div>

      <div className="print-page-toolbar">
        <label className="print-search-label">
          <span>検索</span>
          <input
            type="search"
            value={searchText}
            onChange={(e) => {
              setSearchText(e.target.value)
              setPage(1)
            }}
            placeholder="氏名・住所など"
          />
        </label>
        <div className="print-select-bulk-actions">
          <button
            type="button"
            className="btn btn-label btn-normal"
            onClick={handleSelectAllOk}
            disabled={bulkDisabled || !labelsReady}
          >
            このページのOKを選択
          </button>
          <button
            type="button"
            className="btn btn-label btn-normal"
            onClick={handleDeselectPage}
            disabled={bulkDisabled || !hasSelectionOnPage}
          >
            このページを解除
          </button>
        </div>
      </div>

      {(pruneMessage || bannerError || (excludedAlerts.length > 0 && !bannerError)) && (
        <div className="print-alert" role="status">
          {pruneMessage && <p>{pruneMessage}</p>}
          {bannerError && <p>{bannerError}</p>}
          {!bannerError && excludedAlerts.length > 0 && (
            <p>
              {PRINT_EXCLUDED_BANNER(excludedAlerts.length)}
              <Link to="/addresses">住所録で差出人を紐づけてから再選択 →</Link>
            </p>
          )}
        </div>
      )}

      {error && <p className="print-error">{error}</p>}
      {isLoading && <p className="print-loading">読み込み中…</p>}

      {!isLoading && !error && (
        <table className="print-select-table">
          <thead>
            <tr>
              <th aria-label="選択" />
              <th>氏名</th>
              <th>住所</th>
              <th>差出人（紐づき）</th>
              <th>状態</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => {
              const senderLabel = senderLabels[item.id]
              const known = Object.prototype.hasOwnProperty.call(senderLabels, item.id)
              const excluded = known && senderLabel === null
              const checked = selectedIds.includes(item.id)
              return (
                <tr key={item.id} className={excluded ? 'print-select-row-excluded' : undefined}>
                  <td>
                    <input
                      type="checkbox"
                      checked={checked}
                      disabled={!checked && (!known || excluded || selectedCount >= MAX_PRINT_SELECTION)}
                      onChange={() => handleToggle(item.id, excluded, known)}
                      aria-label={`${formatDisplayName(item.primaryName, item.coRecipients)} を選択`}
                    />
                  </td>
                  <td>
                    {formatDisplayName(item.primaryName, item.coRecipients)}
                    {item.honorific && item.honorific !== 'なし' ? ` ${item.honorific}` : ''}
                    <div className="print-muted">{formatPostalCode(item.postalCode)}</div>
                  </td>
                  <td>{formatAddressSingleLine(item.address)}</td>
                  <td>{known ? (senderLabel ?? '（未紐づけ）') : '…'}</td>
                  <td>{known ? (excluded ? '除外' : 'OK') : '確認中'}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      )}

      {excludedAlerts.length > 0 && (
        <ul className="print-excluded-list">
          {excludedAlerts.map((a) => {
            const row = items.find((item) => item.id === a.addressEntryId)
            const label = row
              ? formatDisplayName(row.primaryName, row.coRecipients)
              : excludedAlertLabel(a)
            return (
              <li key={a.addressEntryId}>
                {label}: {excludedReasonLabel(a.reason)}
              </li>
            )
          })}
        </ul>
      )}

      <div className="print-page-footer">
        <div className="print-pagination">
          <PaginationControls
            currentPage={currentPage}
            totalPages={totalPages}
            onPrev={() => setPage((p) => Math.max(1, p - 1))}
            onNext={() => setPage((p) => Math.min(totalPages, p + 1))}
          />
        </div>
        <div className="print-page-actions">
          <button type="button" className="btn btn-label btn-normal" onClick={handleCancel}>
            キャンセル
          </button>
          <button
            type="button"
            className="btn btn-label btn-primary print-primary-button"
            onClick={handleProceed}
            disabled={resolving || selectedCount === 0}
          >
            {resolving ? '確認中…' : 'プレビューへ進む →'}
          </button>
        </div>
      </div>
    </div>
  )
}
