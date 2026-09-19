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
  /** 単一選択（従来） */
  onSelect?: (item: AddressEntryListItem) => boolean | Promise<boolean> | void
  /** 複数選択モード */
  mode?: 'single' | 'multi'
  initialSelectedIds?: string[]
  onSelectMany?: (items: AddressEntryListItem[]) => void
  /** 複数選択の上限（既定 200） */
  maxSelection?: number
}

const PAGE_SIZE = 10
const DEFAULT_MAX_SELECTION = 200

export function AddressEntrySelectDialog({
  isOpen,
  excludeIds = [],
  onClose,
  onSelect,
  mode = 'single',
  initialSelectedIds = [],
  onSelectMany,
  maxSelection = DEFAULT_MAX_SELECTION,
}: Props) {
  const [keyword, setKeyword] = useState('')
  const [page, setPage] = useState(1)
  const [items, setItems] = useState<AddressEntryListItem[]>([])
  const [total, setTotal] = useState(0)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectingId, setSelectingId] = useState<string | null>(null)
  const [selectedMap, setSelectedMap] = useState<Map<string, AddressEntryListItem>>(() => new Map())
  const [selectionLimitMessage, setSelectionLimitMessage] = useState<string | null>(null)

  const totalPages = useMemo(() => Math.max(1, Math.ceil(total / PAGE_SIZE)), [total])
  const currentPage = Math.min(page, totalPages)
  const excludeSet = useMemo(() => new Set(excludeIds), [excludeIds])
  const visibleItems = useMemo(
    () => items.filter((i) => !excludeSet.has(i.id)),
    [excludeSet, items],
  )
  const initialIdKey = initialSelectedIds.join(',')

  // 開いたとき initialSelectedIds を復元（他ページの ID も get_address_entry で補完）
  useEffect(() => {
    if (!isOpen) return
    if (mode !== 'multi') {
      setSelectedMap(new Map())
      setSelectionLimitMessage(null)
      return
    }
    let cancelled = false
    setSelectionLimitMessage(null)
    if (initialSelectedIds.length === 0) {
      setSelectedMap(new Map())
      return
    }
    ;(async () => {
      const next = new Map<string, AddressEntryListItem>()
      await Promise.all(
        initialSelectedIds.slice(0, maxSelection).map(async (id) => {
          try {
            const dto = await invoke<AddressEntryDto>('get_address_entry', { id })
            if (!cancelled) next.set(id, fromAddressEntryDto(dto))
          } catch {
            // 削除済み等はスキップ
          }
        }),
      )
      if (!cancelled) setSelectedMap(next)
    })()
    return () => {
      cancelled = true
    }
    // initialIdKey で配列内容の変化を検知
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen, mode, initialIdKey, maxSelection])

  // 一覧取得後、選択中 ID の行データを最新化
  useEffect(() => {
    if (!isOpen || mode !== 'multi') return
    setSelectedMap((prev) => {
      if (prev.size === 0) return prev
      let changed = false
      const next = new Map(prev)
      for (const item of items) {
        if (!next.has(item.id)) continue
        next.set(item.id, item)
        changed = true
      }
      return changed ? next : prev
    })
  }, [items, isOpen, mode])

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

  const handleSelect = async (item: AddressEntryListItem) => {
    if (selectingId || !onSelect) return
    setSelectingId(item.id)
    try {
      const shouldClose = await onSelect(item)
      if (shouldClose !== false) {
        onClose()
      }
    } finally {
      setSelectingId(null)
    }
  }

  const toggleMulti = (item: AddressEntryListItem) => {
    setSelectedMap((prev) => {
      const next = new Map(prev)
      if (next.has(item.id)) {
        next.delete(item.id)
        setSelectionLimitMessage(null)
        return next
      }
      if (next.size >= maxSelection) {
        setSelectionLimitMessage(`最大 ${maxSelection} 件まで選択できます。`)
        return prev
      }
      next.set(item.id, item)
      setSelectionLimitMessage(null)
      return next
    })
  }

  const handleConfirmMulti = () => {
    if (!onSelectMany) return
    onSelectMany([...selectedMap.values()].slice(0, maxSelection))
    onClose()
  }

  const selectedCount = selectedMap.size

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

          {mode === 'multi' ? (
            <p className="address-list-meta">
              選択中: {selectedCount} / {maxSelection} 件
            </p>
          ) : null}
          {selectionLimitMessage ? (
            <p className="address-list-error" role="status">
              {selectionLimitMessage}
            </p>
          ) : null}

          {isLoading ? <p className="address-list-loading">読み込み中です…</p> : null}
          {error ? <p className="address-list-error">{error}</p> : null}

          {!isLoading && !error && visibleItems.length === 0 ? (
            <p>該当する宛名がありません。</p>
          ) : null}

          {!isLoading && !error && visibleItems.length > 0 ? (
            <table className="address-list-table" aria-label="宛名選択テーブル">
              <thead>
                <tr>
                  {mode === 'multi' ? <th scope="col">選択</th> : null}
                  <th scope="col">宛名</th>
                  <th scope="col">郵便番号</th>
                  <th scope="col">住所</th>
                  {mode === 'single' ? <th scope="col">操作</th> : null}
                </tr>
              </thead>
              <tbody>
                {visibleItems.map((a) => {
                  const displayName = formatDisplayName(a.primaryName, a.coRecipients)
                  const postal = formatPostalCode(a.postalCode)
                  const address = formatAddressSingleLine(a.address)
                  const checked = selectedMap.has(a.id)
                  return (
                    <tr key={a.id} className="address-list-row">
                      {mode === 'multi' ? (
                        <td>
                          <input
                            type="checkbox"
                            checked={checked}
                            onChange={() => toggleMulti(a)}
                            aria-label={`${displayName} を選択`}
                          />
                        </td>
                      ) : null}
                      <td>
                        <span className="address-list-name">{displayName}</span>
                        <span className="address-list-honorific">{a.honorific}</span>
                      </td>
                      <td className="address-list-postal">{postal || a.postalCode}</td>
                      <td className="address-list-address" title={address}>
                        {address}
                      </td>
                      {mode === 'single' ? (
                        <td className="address-list-actions">
                          <button
                            type="button"
                            disabled={selectingId !== null}
                            onClick={() => {
                              void handleSelect(a)
                            }}
                          >
                            {selectingId === a.id ? '選択中…' : '選択'}
                          </button>
                        </td>
                      ) : null}
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

          {mode === 'multi' ? (
            <div className="dialog-footer-actions">
              <button type="button" className="secondary" onClick={onClose}>
                キャンセル
              </button>
              <button type="button" onClick={handleConfirmMulti} disabled={selectedCount === 0}>
                選択を確定（{selectedCount}）
              </button>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}
