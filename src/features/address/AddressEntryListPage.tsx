import { useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import type { AddressEntryListItem } from './types'
import {
  formatAddressSingleLine,
  formatDisplayName,
  formatPostalCode,
  formatUpdatedAt,
} from './types'
import { useAddressEntryList } from './useAddressEntryList'
import type { ListSortKey, ListSortOrder } from './useAddressEntryList'

type AddressEntryListPageProps = {
  onClickCreate?: () => void
  onSelectDetail?: (id: string) => void
  onClickEdit?: (id: string) => void
  onClickArchive?: (id: string) => void
}

const PAGE_SIZE = 20

export function AddressEntryListPage({
  onClickCreate,
  onSelectDetail,
  onClickEdit,
  onClickArchive,
}: AddressEntryListPageProps) {
  const navigate = useNavigate()
  const [isFilterOpen, setIsFilterOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const [sortKey, setSortKey] = useState<ListSortKey>('nameKana')
  const [sortOrder, setSortOrder] = useState<ListSortOrder>('asc')
  const [page, setPage] = useState(1)

  const { items, isLoading, error, reload } = useAddressEntryList({
    searchText,
    sortKey,
    sortOrder,
  })

  const sortedItems: AddressEntryListItem[] = useMemo(
    () => items.filter((item) => !item.archived),
    [items],
  )

  const totalPages = Math.max(1, Math.ceil(sortedItems.length / PAGE_SIZE))
  const currentPage = Math.min(page, totalPages)
  const startIndex = (currentPage - 1) * PAGE_SIZE
  const pagedItems = sortedItems.slice(startIndex, startIndex + PAGE_SIZE)

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
    if (onSelectDetail) {
      onSelectDetail(id)
      return
    }
    navigate(`/addresses/${id}`)
  }

  const handleClickEdit = (id: string) => {
    if (onClickEdit) {
      onClickEdit(id)
      return
    }
    // eslint-disable-next-line no-alert
    alert('編集画面（ADDR003）は今後実装予定です。')
  }

  const handleClickArchive = (id: string) => {
    if (onClickArchive) {
      onClickArchive(id)
      return
    }
    // eslint-disable-next-line no-alert
    const confirmed = window.confirm(
      'この住所録エントリをアーカイブしますか？一覧からは非表示になりますが、データは保持されます。',
    )
    if (!confirmed) return

    ;(async () => {
      try {
        await invoke('archive_address_entry', { id })
        reload()
      } catch (e) {
        // eslint-disable-next-line no-alert
        alert(String(e))
      }
    })()
  }

  const handlePrevPage = () => {
    setPage((prev) => Math.max(1, prev - 1))
  }

  const handleNextPage = () => {
    setPage((prev) => Math.min(totalPages, prev + 1))
  }

  const hasAnyItems = items.length > 0
  const hasResults = pagedItems.length > 0

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
              if (onClickCreate) {
                onClickCreate()
                return
              }
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
            <option value="nameKana">氏名（カナ）</option>
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

      {!hasAnyItems && (
        <div className="address-list-empty">
          <p>まだ住所録が登録されていません。</p>
          <button
            type="button"
            className="address-list-create-button-primary"
            onClick={onClickCreate}
          >
            新規作成
          </button>
        </div>
      )}

      {hasAnyItems && !hasResults && (
        <div className="address-list-no-results">
          <p>該当する住所録が見つかりませんでした。</p>
          <button type="button" onClick={handleClearSearch}>
            検索条件をクリア
          </button>
        </div>
      )}

      {hasAnyItems && hasResults && (
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

          <div className="address-list-pagination">
            <button
              type="button"
              onClick={handlePrevPage}
              disabled={currentPage === 1}
            >
              前へ
            </button>
            <span className="address-list-page-info">
              {currentPage} / {totalPages}
            </span>
            <button
              type="button"
              onClick={handleNextPage}
              disabled={currentPage === totalPages}
            >
              次へ
            </button>
          </div>
        </>
      )}
    </div>
  )
}

