import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Link, useNavigate } from 'react-router-dom'
import { PaginationControls } from '../../components/PaginationControls'
import { clampPage, totalPagesFor } from '../../lib/pagination'
import type { AddressEntryListItem } from '../address/types'
import { AddressEntrySelectDialog } from '../sender/AddressEntrySelectDialog'
import { POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE } from './messages'
import { usePostcardReceiptList } from './usePostcardReceiptList'
import {
  POSTCARD_RECEIPT_CATEGORY_OPTIONS,
  buildYearOptions,
  categoryLabel,
  formatAddressEntryLabel,
  formatReceivedAt,
  resolveSenderDisplayName,
} from './types'

const PAGE_SIZE = 20

export function PostcardReceiptListPage() {
  const navigate = useNavigate()
  const [isFilterOpen, setIsFilterOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const [year, setYear] = useState('')
  const [category, setCategory] = useState('')
  const [addressEntryId, setAddressEntryId] = useState<string | null>(null)
  const [addressFilterLabel, setAddressFilterLabel] = useState<string | null>(null)
  const [isAddressDialogOpen, setAddressDialogOpen] = useState(false)
  const [page, setPage] = useState(1)
  const [availableYears, setAvailableYears] = useState<number[]>([])
  const [yearsReloadToken, setYearsReloadToken] = useState(0)

  const { items, total, isLoading, error, reload } = usePostcardReceiptList({
    searchText,
    year,
    category,
    addressEntryId,
    page,
    pageSize: PAGE_SIZE,
    onPageChange: setPage,
  })

  useEffect(() => {
    let cancelled = false
    ;(async () => {
      try {
        const years = await invoke<number[]>('list_postcard_receipt_years')
        if (!cancelled) setAvailableYears(years)
      } catch (e) {
        console.error('Failed to load postcard receipt years:', e)
        if (!cancelled) setAvailableYears([])
      }
    })()
    return () => {
      cancelled = true
    }
  }, [yearsReloadToken])

  const yearOptions = useMemo(() => buildYearOptions(availableYears), [availableYears])
  const reloadYears = () => setYearsReloadToken((t) => t + 1)

  const totalPages = totalPagesFor(total, PAGE_SIZE)
  const currentPage = clampPage(page, total, PAGE_SIZE)
  const isFiltering =
    searchText.trim().length > 0 || year !== '' || category !== '' || addressEntryId !== null
  const isNoData = !isFiltering && total === 0
  const isNoSearchResult = isFiltering && total === 0
  const hasItems = total > 0

  const handleClearFilters = () => {
    setSearchText('')
    setYear('')
    setCategory('')
    setAddressEntryId(null)
    setAddressFilterLabel(null)
    setPage(1)
  }

  const handleSelectAddressFilter = (item: AddressEntryListItem) => {
    setAddressEntryId(item.id)
    setAddressFilterLabel(
      formatAddressEntryLabel(item.primaryName, item.coRecipients, item.honorific),
    )
    setAddressDialogOpen(false)
    setPage(1)
  }

  const handleDelete = (id: string) => {
    const confirmed = window.confirm('この受取履歴を削除しますか？一覧からは非表示になります。')
    if (!confirmed) return

    ;(async () => {
      try {
        await invoke('delete_postcard_receipt', { id })
        reload()
        reloadYears()
      } catch (deleteError) {
        console.error('Failed to delete postcard receipt:', deleteError)
        alert(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
      }
    })()
  }

  return (
    <div className="address-list-container">
      <div className="address-list-header">
        <h1 className="address-list-title">受取履歴一覧</h1>
        <div className="address-list-header-actions">
          <button
            type="button"
            className="address-list-filter-toggle"
            onClick={() => setIsFilterOpen((open) => !open)}
          >
            フィルタ
          </button>
          <button
            type="button"
            className="address-list-create-button"
            onClick={() => navigate('/receipts/new')}
          >
            新規作成
          </button>
        </div>
      </div>

      {isFilterOpen ? (
        <div className="address-list-filter">
          <label className="address-list-filter-label">
            <span>検索</span>
            <input
              type="text"
              value={searchText}
              onChange={(e) => {
                setSearchText(e.target.value)
              }}
              placeholder="表示名・メモで検索"
              className="address-list-filter-input"
            />
          </label>

          <label className="address-list-filter-label">
            <span>受取年</span>
            <select
              value={year}
              onChange={(e) => {
                setYear(e.target.value)
                setPage(1)
              }}
            >
              {yearOptions.map((option) => (
                <option key={option.value || 'all'} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>

          <label className="address-list-filter-label">
            <span>種別</span>
            <select
              value={category}
              onChange={(e) => {
                setCategory(e.target.value)
                setPage(1)
              }}
            >
              <option value="">全種別</option>
              {POSTCARD_RECEIPT_CATEGORY_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>

          <div className="address-list-filter-label">
            <span>相手</span>
            <button type="button" onClick={() => setAddressDialogOpen(true)}>
              相手を選択
            </button>
            {addressFilterLabel ? (
              <p>
                選択中: {addressFilterLabel}
                <button
                  type="button"
                  onClick={() => {
                    setAddressEntryId(null)
                    setAddressFilterLabel(null)
                    setPage(1)
                  }}
                >
                  クリア
                </button>
              </p>
            ) : null}
          </div>

          <div className="address-list-filter-actions">
            <button type="button" onClick={handleClearFilters} disabled={!isFiltering}>
              条件クリア
            </button>
          </div>
        </div>
      ) : null}

      {isLoading ? <p className="address-list-loading">読み込み中です…</p> : null}
      {error ? <p className="address-list-error">一覧の取得に失敗しました: {error}</p> : null}

      {!isLoading && !error && isNoData ? (
        <div className="address-list-empty">
          <p>まだ受取履歴が登録されていません。</p>
          <button type="button" className="address-list-create-button-primary" onClick={() => navigate('/receipts/new')}>
            新規作成
          </button>
        </div>
      ) : null}

      {!isLoading && !error && isNoSearchResult ? (
        <div className="address-list-no-results">
          <p>該当する受取履歴が見つかりませんでした。</p>
          <button type="button" onClick={handleClearFilters}>
            条件クリア
          </button>
        </div>
      ) : null}

      {!isLoading && !error && hasItems ? (
        <>
          <table className="address-list-table" aria-label="受取履歴一覧テーブル">
            <thead>
              <tr>
                <th scope="col">受取日</th>
                <th scope="col">種別</th>
                <th scope="col">送り主</th>
                <th scope="col">メモ（抜粋）</th>
                <th scope="col">操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => {
                const senderName = resolveSenderDisplayName(item)
                const memoSnippet = (item.memo ?? '').slice(0, 30)
                return (
                  <tr key={item.id} className="address-list-row">
                    <td>{formatReceivedAt(item.receivedAt)}</td>
                    <td>{categoryLabel(item.category)}</td>
                    <td>
                      <Link to={`/receipts/${item.id}`} className="address-list-name">
                        {senderName}
                      </Link>
                    </td>
                    <td className="address-list-memo" title={item.memo ?? ''}>
                      {memoSnippet}
                      {item.memo && item.memo.length > memoSnippet.length ? '…' : ''}
                    </td>
                    <td className="address-list-actions">
                      <button type="button" onClick={() => navigate(`/receipts/${item.id}/edit`)}>
                        編集
                      </button>
                      <button type="button" onClick={() => handleDelete(item.id)}>
                        削除
                      </button>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>

          <PaginationControls
            currentPage={currentPage}
            totalPages={totalPages}
            onPrev={() => setPage((prev) => Math.max(1, prev - 1))}
            onNext={() => setPage((prev) => Math.min(totalPages, prev + 1))}
          />
        </>
      ) : null}

      {isAddressDialogOpen ? (
        <AddressEntrySelectDialog
          isOpen={isAddressDialogOpen}
          onClose={() => setAddressDialogOpen(false)}
          onSelect={handleSelectAddressFilter}
        />
      ) : null}
    </div>
  )
}
