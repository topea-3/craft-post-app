import { useEffect, useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { resolvePrintJobItems } from '../api'
import { usePrintJobDraft } from '../hooks/usePrintJobDraft'
import {
  PRINT_NO_VALID_ITEMS_MESSAGE,
  PRINT_OPERATION_ERROR_MESSAGE,
  PRINT_RESOLVE_INVALID_MESSAGE,
} from '../messages'
import type { ExcludedAlert, PrintJobItem } from '../types'
import {
  excludedAlertLabel,
  excludedReasonLabel,
  formatRecipientDisplayName,
  formatSenderDisplayNameFromSnapshot,
  parseAddressEntriesInvalidError,
} from '../types'

type LocationState = {
  items?: PrintJobItem[]
  excludedAlerts?: ExcludedAlert[]
}

export function PrintConfirmPage() {
  const navigate = useNavigate()
  const location = useLocation()
  const state = (location.state ?? {}) as LocationState
  const { selectedIds, removeIds, setExcludedAlerts, clearDraft } = usePrintJobDraft()

  const [items, setItems] = useState<PrintJobItem[]>(state.items ?? [])
  const [excluded, setExcluded] = useState<ExcludedAlert[]>(state.excludedAlerts ?? [])
  const [loading, setLoading] = useState(!state.items)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (selectedIds.length === 0) {
      navigate('/print/select', { replace: true })
      return
    }
    if (state.items && state.items.length > 0) {
      return
    }

    let cancelled = false
    ;(async () => {
      setLoading(true)
      setError(null)
      try {
        const result = await resolvePrintJobItems(selectedIds)
        if (cancelled) return
        if (result.items.length === 0) {
          setExcludedAlerts(result.excludedAlerts)
          setError(PRINT_NO_VALID_ITEMS_MESSAGE)
          setItems([])
          setExcluded(result.excludedAlerts)
          return
        }
        setItems(result.items)
        setExcluded(result.excludedAlerts)
        setExcludedAlerts(result.excludedAlerts)
      } catch (e) {
        if (cancelled) return
        const invalid = parseAddressEntriesInvalidError(e)
        if (invalid) {
          removeIds(invalid.entries.map((x) => x.address_entry_id))
          setError(PRINT_RESOLVE_INVALID_MESSAGE)
          navigate('/print/select')
          return
        }
        console.error('resolve on confirm failed:', e)
        setError(PRINT_OPERATION_ERROR_MESSAGE)
      } finally {
        if (!cancelled) setLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [
    navigate,
    removeIds,
    selectedIds,
    setExcludedAlerts,
    state.items,
  ])

  const handleBack = () => {
    navigate('/print/select')
  }

  const handleCancel = () => {
    clearDraft()
    navigate('/addresses')
  }

  const handleNext = () => {
    if (items.length === 0) return
    navigate('/print/preview', { state: { items, excludedAlerts: excluded } })
  }

  return (
    <div className="print-page">
      <div className="print-page-header">
        <h1 className="print-page-title">印刷内容の確認</h1>
      </div>

      <p>以下の宛名と差出人で印刷します。（差出人の変更はできません）</p>

      {loading && <p className="print-loading">読み込み中…</p>}
      {error && <p className="print-error">{error}</p>}

      {!loading && items.length > 0 && (
        <table className="print-select-table">
          <thead>
            <tr>
              <th>宛名</th>
              <th>差出人（紐づき）</th>
              <th>住所（抜粋）</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <tr key={item.address.addressEntryId}>
                <td>{formatRecipientDisplayName(item.address)}</td>
                <td>{formatSenderDisplayNameFromSnapshot(item.sender)}</td>
                <td>{item.address.addressLine1}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {excluded.length > 0 && (
        <div className="print-alert">
          <p>除外された宛名（参考）:</p>
          <ul>
            {excluded.map((a) => (
              <li key={a.addressEntryId}>
                {excludedAlertLabel(a)} — {excludedReasonLabel(a.reason)}
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="print-page-actions">
        <button type="button" onClick={handleCancel}>
          キャンセル
        </button>
        <button type="button" onClick={handleBack}>
          戻る
        </button>
        <button
          type="button"
          className="print-primary-button"
          onClick={handleNext}
          disabled={items.length === 0 || loading}
        >
          プレビューへ →
        </button>
      </div>
    </div>
  )
}
