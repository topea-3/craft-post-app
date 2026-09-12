import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import { PaginationControls } from '../../components/PaginationControls'
import { formatAddressSingleLine, formatPostalCode, formatUpdatedAt } from '../address/types'
import { SENDER_OPERATION_ERROR_MESSAGE } from './messages'
import { formatSenderDisplayName } from './types'
import { useSenderEntryList } from './useSenderEntryList'

export function SenderEntryListPage() {
  const navigate = useNavigate()
  const [page, setPage] = useState(1)
  const PAGE_SIZE = 20

  const { items, isLoading, error, hasNext, reload } = useSenderEntryList({
    page,
    pageSize: PAGE_SIZE,
  })

  const handleArchive = (id: string) => {
    const confirmed = window.confirm(
      'この差出人をアーカイブしますか？一覧からは非表示になりますが、データは保持されます。',
    )
    if (!confirmed) return
    ;(async () => {
      try {
        await invoke('archive_sender_entry', { id })
        reload()
      } catch (e) {
        console.error('Failed to archive sender entry:', e)
        alert(SENDER_OPERATION_ERROR_MESSAGE)
      }
    })()
  }

  return (
    <div className="address-list-container">
      <div className="address-list-header">
        <h1 className="address-list-title">差出人一覧</h1>
        <button
          type="button"
          className="address-list-create-button"
          onClick={() => {
            navigate('/senders/new')
          }}
        >
          新規作成
        </button>
      </div>

      {isLoading ? <p className="address-list-loading">読み込み中です…</p> : null}
      {error ? <p className="address-list-error">一覧の取得に失敗しました: {error}</p> : null}

      {!isLoading && !error && items.length === 0 && (
        <div className="address-list-empty">
          <p>まだ差出人が登録されていません。</p>
          <button
            type="button"
            className="address-list-create-button-primary"
            onClick={() => {
              navigate('/senders/new')
            }}
          >
            新規作成
          </button>
        </div>
      )}

      {!isLoading && !error && items.length > 0 && (
        <>
          <table className="address-list-table" aria-label="差出人一覧テーブル">
            <thead>
              <tr>
                <th scope="col">ラベル</th>
                <th scope="col">差出人（表示名）</th>
                <th scope="col">郵便番号</th>
                <th scope="col">住所</th>
                <th scope="col">最終更新日時</th>
                <th scope="col">操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => {
                const displayName = formatSenderDisplayName(item.primaryName, item.coRecipients)
                const postalCode = formatPostalCode(item.postalCode)
                const addressLine = formatAddressSingleLine(item.address)
                const updatedAt = formatUpdatedAt(item.updatedAt)
                return (
                  <tr
                    key={item.id}
                    className="address-list-row"
                    onClick={() => {
                      navigate(`/senders/${item.id}`)
                    }}
                  >
                    <td>
                      <span className="address-list-name">{item.label}</span>
                    </td>
                    <td>{displayName}</td>
                    <td className="address-list-postal">{postalCode || item.postalCode}</td>
                    <td className="address-list-address" title={addressLine}>
                      {addressLine}
                    </td>
                    <td className="address-list-updated-at">{updatedAt}</td>
                    <td className="address-list-actions">
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation()
                          navigate(`/senders/${item.id}/edit`)
                        }}
                      >
                        編集
                      </button>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleArchive(item.id)
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
            currentPage={page}
            hasNext={hasNext}
            onPrev={() => {
              setPage((prev) => Math.max(1, prev - 1))
            }}
            onNext={() => {
              setPage((prev) => prev + 1)
            }}
          />
        </>
      )}
    </div>
  )
}

