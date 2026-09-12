import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { PaginationControls } from '../../components/PaginationControls'
import { formatAddressSingleLine, formatPostalCode } from '../address/types'
import {
  formatSenderDisplayName,
  formatSenderOptionLabel,
  fromSenderEntryDtoToDetail,
  type SenderEntryDto,
  type SenderEntryListItem,
} from './types'
import { useSenderEntryList } from './useSenderEntryList'

type Props = {
  isOpen: boolean
  selectedId?: string | null
  onClose: () => void
  onSelect: (senderEntryId: string) => void
}

const PAGE_SIZE = 10

export function SenderEntrySelectDialog({ isOpen, selectedId, onClose, onSelect }: Props) {
  if (!isOpen) return null

  return (
    <SenderEntrySelectDialogContent
      selectedId={selectedId}
      onClose={onClose}
      onSelect={onSelect}
    />
  )
}

type ContentProps = {
  selectedId?: string | null
  onClose: () => void
  onSelect: (senderEntryId: string) => void
}

function SenderEntrySelectDialogContent({ selectedId, onClose, onSelect }: ContentProps) {
  const [page, setPage] = useState(1)
  const [fetchedSelected, setFetchedSelected] = useState<{
    id: string
    sender: SenderEntryListItem
  } | null>(null)

  const { items, isLoading, error, hasNext } = useSenderEntryList({
    page,
    pageSize: PAGE_SIZE,
  })

  const selectedInPage = useMemo(
    () => (selectedId ? items.find((item) => item.id === selectedId) ?? null : null),
    [items, selectedId],
  )

  const needsOutsideFetch = Boolean(
    selectedId && !items.some((item) => item.id === selectedId),
  )

  useEffect(() => {
    if (!needsOutsideFetch || !selectedId) {
      return
    }

    let cancelled = false
    const fetchSelected = async () => {
      try {
        const dto = await invoke<SenderEntryDto>('get_sender_entry', { id: selectedId })
        if (cancelled) return
        setFetchedSelected({ id: selectedId, sender: fromSenderEntryDtoToDetail(dto) })
      } catch (e) {
        if (cancelled) return
        console.error('Failed to fetch selected sender entry:', e)
        setFetchedSelected(null)
      }
    }

    fetchSelected()
    return () => {
      cancelled = true
    }
  }, [needsOutsideFetch, selectedId])

  const highlightedOutsidePage =
    fetchedSelected && fetchedSelected.id === selectedId ? fetchedSelected.sender : null
  const highlightedSender = selectedInPage ?? highlightedOutsidePage

  return (
    <div className="dialog-overlay" role="dialog" aria-modal="true">
      <div className="dialog-panel">
        <div className="dialog-header">
          <h2 className="dialog-title">差出人を選択</h2>
          <button type="button" className="secondary" onClick={onClose}>
            閉じる
          </button>
        </div>

        <div className="dialog-body">
          {highlightedSender ? (
            <div className="sender-select-current">
              <p className="sender-select-current-label">現在の選択</p>
              <p className="sender-select-current-value">{formatSenderOptionLabel(highlightedSender)}</p>
            </div>
          ) : null}

          {isLoading ? <p className="address-list-loading">読み込み中です…</p> : null}
          {error ? <p className="address-list-error">{error}</p> : null}

          {!isLoading && !error && items.length === 0 ? <p>差出人が登録されていません。</p> : null}

          {!isLoading && !error && items.length > 0 ? (
            <table className="address-list-table" aria-label="差出人選択テーブル">
              <thead>
                <tr>
                  <th scope="col">ラベル</th>
                  <th scope="col">差出人（表示名）</th>
                  <th scope="col">郵便番号</th>
                  <th scope="col">住所</th>
                  <th scope="col">操作</th>
                </tr>
              </thead>
              <tbody>
                {items.map((sender) => {
                  const displayName = formatSenderDisplayName(sender.primaryName, sender.coRecipients)
                  const postal = formatPostalCode(sender.postalCode)
                  const address = formatAddressSingleLine(sender.address)
                  const isSelected = sender.id === selectedId
                  return (
                    <tr
                      key={sender.id}
                      className={`address-list-row${isSelected ? ' sender-select-row-selected' : ''}`}
                    >
                      <td>
                        <span className="address-list-name">{sender.label}</span>
                      </td>
                      <td>{displayName || '—'}</td>
                      <td className="address-list-postal">{postal || sender.postalCode}</td>
                      <td className="address-list-address" title={address}>
                        {address}
                      </td>
                      <td className="address-list-actions">
                        <button
                          type="button"
                          onClick={() => {
                            onSelect(sender.id)
                          }}
                        >
                          選択
                        </button>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          ) : null}

          <PaginationControls
            currentPage={page}
            hasNext={hasNext}
            onPrev={() => {
              setPage((prev) => Math.max(1, prev - 1))
            }}
            onNext={() => {
              setPage((prev) => prev + 1)
            }}
          />
        </div>
      </div>
    </div>
  )
}
