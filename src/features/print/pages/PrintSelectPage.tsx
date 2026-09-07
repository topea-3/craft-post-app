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
  PRINT_SELECT_MAX_MESSAGE,
} from '../messages'
import {
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

  const handleToggle = (id: string, excluded: boolean) => {
    const checked = selectedIds.includes(id)
    // 除外行でも既選択なら解除を許可（ラベル遅延到着でロックされるのを防ぐ）
    if (excluded && !checked) return
    if (!checked && selectedIds.length >= MAX_PRINT_SELECTION) {
      setBannerError(PRINT_SELECT_MAX_MESSAGE)
      return
    }
    setBannerError(null)
    toggleId(id)
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
                      disabled={!checked && (excluded || selectedCount >= MAX_PRINT_SELECTION)}
                      onChange={() => handleToggle(item.id, excluded)}
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
                  <td>{excluded ? '除外' : 'OK'}</td>
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
              : a.addressEntryId
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
          <button type="button" onClick={handleCancel}>
            キャンセル
          </button>
          <button
            type="button"
            className="print-primary-button"
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
