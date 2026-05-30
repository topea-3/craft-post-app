import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import { PaginationControls } from '../../components/PaginationControls'
import {
  formatAddressSingleLine,
  formatDisplayName,
  formatPostalCode,
  formatUpdatedAt,
} from './types'
import { ADDRESS_OPERATION_ERROR_MESSAGE } from './messages'
import { useAddressEntryList } from './useAddressEntryList'
import type { ListSortKey, ListSortOrder } from './useAddressEntryList'

const PAGE_SIZE = 20

export function AddressEntryListPage() {
  const navigate = useNavigate()
  const [isFilterOpen, setIsFilterOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const [sortKey, setSortKey] = useState<ListSortKey>('nameKana')
  const [sortOrder, setSortOrder] = useState<ListSortOrder>('asc')
  const [page, setPage] = useState(1)

  const { items, total, isLoading, error, reload } = useAddressEntryList({
    searchText,
    sortKey,
    sortOrder,
    page,
    pageSize: PAGE_SIZE,
  })

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))
  const currentPage = Math.min(page, totalPages)

  const pagedItems = items

  const handleChangeSortKey = (value: ListSortKey) => {
    setSortKey(value)
    setPage(1)
  }

  const handleChangeSortOrder = (value: ListSortOrder) => {
    setSortOrder(value)
    setPage(1)
  }

  const handleClearSearch = () => {
    setSearchText('')
    setPage(1)
  }

  const handleClickRow = (id: string) => {
    navigate(`/addresses/${id}`)
  }

  const handleClickEdit = (id: string) => {
    navigate(`/addresses/${id}/edit`)
  }

  const handleClickArchive = (id: string) => {
    const confirmed = window.confirm(
      'この住所録エントリをアーカイブしますか？一覧からは非表示になりますが、データは保持されます。',
    )
    if (!confirmed) return

    ;(async () => {
      try {
        await invoke('archive_address_entry', { id })
        reload()
      } catch (error) {
        console.error('Failed to archive address entry:', error)
        alert(ADDRESS_OPERATION_ERROR_MESSAGE)
      }
    })()
  }

  const handlePrevPage = () => {
    setPage((prev) => Math.max(1, prev - 1))
  }

  const handleNextPage = () => {
    setPage((prev) => Math.min(totalPages, prev + 1))
  }

  const isFiltering = searchText.trim().length > 0
  const isNoData = !isFiltering && total === 0
  const isNoSearchResult = isFiltering && total === 0
  const hasItems = total > 0

  return (
    <div className="address-list-container">
      <div className="address-list-header">
        <h1 className="address-list-title">住所録一覧</h1>
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
            onClick={() => {
              navigate('/addresses/new')
            }}
          >
            新規作成
          </button>
        </div>
      </div>

      {isFilterOpen && (
        <div className="address-list-filter">
          <label className="address-list-filter-label">
            <span>検索</span>
            <input
              type="text"
              value={searchText}
              onChange={(e) => {
                setSearchText(e.target.value)
                setPage(1)
              }}
              placeholder="氏名・住所・メモで検索"
              className="address-list-filter-input"
            />
          </label>
          <div className="address-list-filter-actions">
            <button
              type="button"
              onClick={handleClearSearch}
              disabled={!searchText}
            >
              条件クリア
            </button>
          </div>
        </div>
      )}

      <div className="address-list-sort">
        <label>
          並び替え:
          <select value={sortKey} onChange={(e) => handleChangeSortKey(e.target.value as ListSortKey)}>
            <option value="nameKana">氏名</option>
            <option value="updatedAt">最終更新日時</option>
          </select>
        </label>
        <label>
          順序:
          <select value={sortOrder} onChange={(e) => handleChangeSortOrder(e.target.value as ListSortOrder)}>
            <option value="asc">昇順</option>
            <option value="desc">降順</option>
          </select>
        </label>
      </div>

      {isLoading && (
        <p className="address-list-loading">読み込み中です…</p>
      )}

      {error && (
        <p className="address-list-error">
          一覧の取得に失敗しました: {error}
        </p>
      )}

      {!isLoading && !error && isNoData && (
        <div className="address-list-empty">
          <p>まだ住所録が登録されていません。</p>
          <button
            type="button"
            className="address-list-create-button-primary"
            onClick={() => {
              navigate('/addresses/new')
            }}
          >
            新規作成
          </button>
        </div>
      )}

      {!isLoading && !error && isNoSearchResult && (
        <div className="address-list-no-results">
          <p>該当する住所録が見つかりませんでした。</p>
          <button type="button" onClick={handleClearSearch}>
            検索条件をクリア
          </button>
        </div>
      )}

      {!isLoading && !error && hasItems && (
        <>
          <table
            className="address-list-table"
            aria-label="住所録一覧テーブル"
          >
            <thead>
              <tr>
                <th scope="col">氏名</th>
                <th scope="col">郵便番号</th>
                <th scope="col">住所</th>
                <th scope="col">メモ（抜粋）</th>
                <th scope="col">最終更新日時</th>
                <th scope="col">操作</th>
              </tr>
            </thead>
            <tbody>
              {pagedItems.map((item) => {
                const displayName = formatDisplayName(
                  item.primaryName,
                  item.coRecipients,
                )
                const postalCode = formatPostalCode(item.postalCode)
                const addressLine = formatAddressSingleLine(item.address)
                const memoSnippet = (item.memo ?? '').slice(0, 30)
                const updatedAt = formatUpdatedAt(item.updatedAt)

                return (
                  <tr
                    key={item.id}
                    className="address-list-row"
                    onClick={() => handleClickRow(item.id)}
                  >
                    <td>
                      <span className="address-list-name">{displayName}</span>
                      <span className="address-list-honorific">
                        {item.honorific}
                      </span>
                    </td>
                    <td className="address-list-postal">
                      {postalCode || item.postalCode}
                    </td>
                    <td
                      className="address-list-address"
                      title={addressLine}
                    >
                      {addressLine}
                    </td>
                    <td
                      className="address-list-memo"
                      title={item.memo ?? ''}
                    >
                      {memoSnippet}
                      {item.memo && item.memo.length > memoSnippet.length
                        ? '…'
                        : ''}
                    </td>
                    <td className="address-list-updated-at">{updatedAt}</td>
                    <td className="address-list-actions">
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleClickEdit(item.id)
                        }}
                      >
                        編集
                      </button>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleClickArchive(item.id)
                        }}
                      >
                        アーカイブ
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
            onPrev={handlePrevPage}
            onNext={handleNextPage}
          />
        </>
      )}
    </div>
  )
}

