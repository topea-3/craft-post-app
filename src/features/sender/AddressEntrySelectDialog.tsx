import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { PaginationControls } from '../../components/PaginationControls'
import type { AddressEntryDto } from '../address/types'
import { fromAddressEntryDto, formatAddressSingleLine, formatDisplayName, formatPostalCode } from '../address/types'
import type { AddressEntryListItem } from '../address/types'
import { ADDRESS_OPERATION_ERROR_MESSAGE } from '../address/messages'

type Props = {
  isOpen: boolean
  excludeIds?: string[]
  onClose: () => void
  onSelect: (item: AddressEntryListItem) => void
}

const PAGE_SIZE = 10

export function AddressEntrySelectDialog({ isOpen, excludeIds = [], onClose, onSelect }: Props) {
  const [keyword, setKeyword] = useState('')
  const [page, setPage] = useState(1)
  const [items, setItems] = useState<AddressEntryListItem[]>([])
  const [total, setTotal] = useState(0)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const totalPages = useMemo(() => Math.max(1, Math.ceil(total / PAGE_SIZE)), [total])
  const currentPage = Math.min(page, totalPages)
  const excludeSet = useMemo(() => new Set(excludeIds), [excludeIds])
  const visibleItems = useMemo(
    () => items.filter((i) => !excludeSet.has(i.id)),
    [excludeSet, items],
  )

  useEffect(() => {
    if (!isOpen) return
    let cancelled = false
    const fetch = async () => {
      setIsLoading(true)
      try {
        const limit = PAGE_SIZE
        const offset = (currentPage - 1) * PAGE_SIZE
        const result = await invoke<{ items: AddressEntryDto[]; total: number }>(
          'search_address_entries',
          {
            keyword: keyword.trim() || null,
            sortKey: 'updated_at',
            sortOrder: 'desc',
            includeArchived: false,
            limit,
            offset,
          },
        )
        if (cancelled) return
        setItems(result.items.map(fromAddressEntryDto))
        setTotal(result.total)
        setError(null)
      } catch (e) {
        if (cancelled) return
        console.error('Failed to search address entries:', e)
        setItems([])
        setTotal(0)
        setError(ADDRESS_OPERATION_ERROR_MESSAGE)
      } finally {
        if (!cancelled) setIsLoading(false)
      }
    }
    fetch()
    return () => {
      cancelled = true
    }
  }, [isOpen, keyword, currentPage])

  useEffect(() => {
    if (!isOpen) return
    setPage(1)
  }, [isOpen, keyword])

  if (!isOpen) return null

  return (
    <div className="dialog-overlay" role="dialog" aria-modal="true">
      <div className="dialog-panel">
        <div className="dialog-header">
          <h2 className="dialog-title">宛名を選択</h2>
          <button type="button" className="secondary" onClick={onClose}>
            閉じる
          </button>
        </div>

        <div className="dialog-body">
          <label className="address-list-filter-label">
            <span>検索</span>
            <input
              className="address-list-filter-input"
              type="text"
              value={keyword}
              onChange={(e) => setKeyword(e.target.value)}
              placeholder="氏名・住所・メモで検索"
            />
          </label>

          {isLoading ? <p className="address-list-loading">読み込み中です…</p> : null}
          {error ? <p className="address-list-error">{error}</p> : null}

          {!isLoading && !error && visibleItems.length === 0 ? (
            <p>該当する宛名がありません。</p>
          ) : null}

          {!isLoading && !error && visibleItems.length > 0 ? (
            <table className="address-list-table" aria-label="宛名選択テーブル">
              <thead>
                <tr>
                  <th scope="col">宛名</th>
                  <th scope="col">郵便番号</th>
                  <th scope="col">住所</th>
                  <th scope="col">操作</th>
                </tr>
              </thead>
              <tbody>
                {visibleItems.map((a) => {
                  const displayName = formatDisplayName(a.primaryName, a.coRecipients)
                  const postal = formatPostalCode(a.postalCode)
                  const address = formatAddressSingleLine(a.address)
                  return (
                    <tr key={a.id} className="address-list-row">
                      <td>
                        <span className="address-list-name">{displayName}</span>
                        <span className="address-list-honorific">{a.honorific}</span>
                      </td>
                      <td className="address-list-postal">{postal || a.postalCode}</td>
                      <td className="address-list-address" title={address}>
                        {address}
                      </td>
                      <td className="address-list-actions">
                        <button
                          type="button"
                          onClick={() => {
                            onSelect(a)
                            onClose()
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
            currentPage={currentPage}
            totalPages={totalPages}
            onPrev={() => setPage((p) => Math.max(1, p - 1))}
            onNext={() => setPage((p) => Math.min(totalPages, p + 1))}
          />
        </div>
      </div>
    </div>
  )
}

