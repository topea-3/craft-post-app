import { useMemo, useState } from 'react'
import type { AddressEntryListItem } from './types'
import {
  formatAddressSingleLine,
  formatDisplayName,
  formatPostalCode,
  formatUpdatedAt,
} from './types'

type SortKey = 'nameKana' | 'updatedAt'
type SortOrder = 'asc' | 'desc'

type AddressEntryListPageProps = {
  onClickCreate: () => void
  onSelectDetail?: (id: string) => void
  onClickEdit?: (id: string) => void
  onClickArchive?: (id: string) => void
}

const PAGE_SIZE = 20

const MOCK_ITEMS: AddressEntryListItem[] = [
  {
    id: '1',
    primaryName: { last: '山田', first: '太郎', kanaLast: 'ヤマダ', kanaFirst: 'タロウ' },
    coRecipients: [],
    honorific: '様',
    postalCode: '1500001',
    address: {
      postalCode: '1500001',
      prefecture: '東京都',
      city: '渋谷区',
      street: '神南 1-1-1',
      building: '○○ビル 3F',
    },
    memo: '会社の取引先。年賀状のみ送付。',
    updatedAt: '2026-03-10T08:34:00+09:00',
    archived: false,
  },
  {
    id: '2',
    primaryName: { last: '佐藤', first: '花子', kanaLast: 'サトウ', kanaFirst: 'ハナコ' },
    coRecipients: [
      { last: '佐藤', first: '一郎' },
      { last: '佐藤', first: '二郎' },
    ],
    honorific: 'ご家族様',
    postalCode: '9800001',
    address: {
      postalCode: '9800001',
      prefecture: '宮城県',
      city: '仙台市青葉区',
      street: '中央 1-2-3',
      building: '',
    },
    memo: '友人家族。暑中見舞いも送付。',
    updatedAt: '2026-03-11T12:01:00+09:00',
    archived: false,
  },
  {
    id: '3',
    primaryName: { last: '株式会社', first: 'サンプル印刷' },
    coRecipients: [],
    honorific: '御中',
    postalCode: '1010001',
    address: {
      postalCode: '1010001',
      prefecture: '東京都',
      city: '千代田区',
      street: '神田 1-2-3',
      building: 'ビルディング 10F',
    },
    memo: '印刷会社。取引停止のためアーカイブ予定。',
    updatedAt: '2026-03-09T09:00:00+09:00',
    archived: true,
  },
]

export function AddressEntryListPage({
  onClickCreate,
  onSelectDetail,
  onClickEdit,
  onClickArchive,
}: AddressEntryListPageProps) {
  const [isFilterOpen, setIsFilterOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const [sortKey, setSortKey] = useState<SortKey>('nameKana')
  const [sortOrder, setSortOrder] = useState<SortOrder>('asc')
  const [page, setPage] = useState(1)

  const activeItems = useMemo(
    () => MOCK_ITEMS.filter((item) => !item.archived),
    [],
  )

  const filteredItems = useMemo(() => {
    if (!searchText.trim()) return activeItems

    const keyword = searchText.trim()
    return activeItems.filter((item) => {
      const displayName = formatDisplayName(item.primaryName, item.coRecipients)
      const addressLine = formatAddressSingleLine(item.address)
      const memo = item.memo ?? ''
      return (
        displayName.includes(keyword) ||
        addressLine.includes(keyword) ||
        memo.includes(keyword)
      )
    })
  }, [activeItems, searchText])

  const sortedItems = useMemo(() => {
    const items = [...filteredItems]
    items.sort((a, b) => {
      if (sortKey === 'nameKana') {
        const aKana = `${a.primaryName.kanaLast ?? ''}${a.primaryName.kanaFirst ?? ''}${
          a.primaryName.last
        }${a.primaryName.first}`
        const bKana = `${b.primaryName.kanaLast ?? ''}${b.primaryName.kanaFirst ?? ''}${
          b.primaryName.last
        }${b.primaryName.first}`
        if (aKana < bKana) return sortOrder === 'asc' ? -1 : 1
        if (aKana > bKana) return sortOrder === 'asc' ? 1 : -1
        return 0
      }

      const aTime = new Date(a.updatedAt).getTime()
      const bTime = new Date(b.updatedAt).getTime()
      if (aTime < bTime) return sortOrder === 'asc' ? -1 : 1
      if (aTime > bTime) return sortOrder === 'asc' ? 1 : -1
      return 0
    })
    return items
  }, [filteredItems, sortKey, sortOrder])

  const totalPages = Math.max(1, Math.ceil(sortedItems.length / PAGE_SIZE))
  const currentPage = Math.min(page, totalPages)
  const startIndex = (currentPage - 1) * PAGE_SIZE
  const pagedItems = sortedItems.slice(startIndex, startIndex + PAGE_SIZE)

  const handleChangeSortKey = (value: SortKey) => {
    setSortKey(value)
    setPage(1)
  }

  const handleChangeSortOrder = (value: SortOrder) => {
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
    }
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
    alert(`ID: ${id} の住所録をアーカイブする処理は今後実装予定です。`)
  }

  const handlePrevPage = () => {
    setPage((prev) => Math.max(1, prev - 1))
  }

  const handleNextPage = () => {
    setPage((prev) => Math.min(totalPages, prev + 1))
  }

  const hasAnyItems = activeItems.length > 0
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
            onClick={onClickCreate}
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
          <select
            value={sortKey}
            onChange={(e) => handleChangeSortKey(e.target.value as SortKey)}
          >
            <option value="nameKana">氏名（カナ）</option>
            <option value="updatedAt">最終更新日時</option>
          </select>
        </label>
        <label>
          順序:
          <select
            value={sortOrder}
            onChange={(e) =>
              handleChangeSortOrder(e.target.value as SortOrder)
            }
          >
            <option value="asc">昇順</option>
            <option value="desc">降順</option>
          </select>
        </label>
      </div>

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

