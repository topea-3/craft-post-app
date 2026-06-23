import { useEffect, useState } from 'react'
import { useLocation, useNavigate, useParams } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import type { PostcardReceiptDto } from './types'
import {
  categoryLabel,
  formatDateTime,
  formatReceivedAt,
  fromPostcardReceiptDtoToDetail,
  resolveSenderDisplayName,
} from './types'
import { POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE } from './messages'

export function PostcardReceiptDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const location = useLocation()
  const [entry, setEntry] = useState<ReturnType<typeof fromPostcardReceiptDtoToDetail> | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [flash, setFlash] = useState<string | null>(
    (location.state as { flash?: string } | null)?.flash ?? null,
  )

  useEffect(() => {
    if (!id) {
      setError('ID が指定されていません。')
      return
    }

    let cancelled = false
    const fetchDetail = async () => {
      setIsLoading(true)
      try {
        const dto = await invoke<PostcardReceiptDto>('get_postcard_receipt', { id })
        if (cancelled) return
        setEntry(fromPostcardReceiptDtoToDetail(dto))
        setError(null)
      } catch (fetchError) {
        if (cancelled) return
        console.error('Failed to fetch postcard receipt detail:', fetchError)
        setError(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
      } finally {
        if (!cancelled) setIsLoading(false)
      }
    }

    fetchDetail()
    return () => {
      cancelled = true
    }
  }, [id])

  const handleBackToList = () => {
    navigate('/receipts')
  }

  const handleEdit = () => {
    if (!id) return
    navigate(`/receipts/${id}/edit`)
  }

  const handleDelete = async () => {
    if (!entry) return
    const confirmed = window.confirm('この受取履歴を削除しますか？一覧からは非表示になります。')
    if (!confirmed) return

    try {
      await invoke('delete_postcard_receipt', { id: entry.id })
      navigate('/receipts')
    } catch (deleteError) {
      console.error('Failed to delete postcard receipt:', deleteError)
      alert(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
    }
  }

  if (isLoading) {
    return (
      <div className="address-detail-container">
        <p>読み込み中です…</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="address-detail-container">
        <p className="address-detail-error">詳細の取得に失敗しました: {error}</p>
        <button type="button" onClick={handleBackToList}>
          一覧に戻る
        </button>
      </div>
    )
  }

  if (!entry) {
    return (
      <div className="address-detail-container">
        <p>受取履歴が見つかりませんでした。</p>
        <button type="button" onClick={handleBackToList}>
          一覧に戻る
        </button>
      </div>
    )
  }

  const senderName = resolveSenderDisplayName(entry)

  return (
    <div className="address-detail-container">
      {flash ? (
        <p className="address-detail-flash" role="status">
          {flash}
          <button type="button" onClick={() => setFlash(null)}>
            閉じる
          </button>
        </p>
      ) : null}

      <header className="address-detail-header">
        <div>
          <h1 className="address-detail-title">受取履歴詳細</h1>
          <div className="address-detail-subtitle">
            <span className="address-detail-name">{senderName}</span>
          </div>
        </div>
        <div className="address-detail-header-actions">
          <button type="button" className="address-list-create-button" onClick={handleEdit}>
            編集
          </button>
          <button type="button" onClick={handleDelete}>
            削除
          </button>
          <button type="button" onClick={handleBackToList}>
            戻る
          </button>
        </div>
      </header>

      <main className="address-detail-main">
        <section className="address-detail-section">
          <h2 className="address-detail-section-title">受取情報</h2>
          <dl className="address-detail-grid">
            <div className="address-detail-row">
              <dt>受取日</dt>
              <dd>{formatReceivedAt(entry.receivedAt)}</dd>
            </div>
            <div className="address-detail-row">
              <dt>種別</dt>
              <dd>{categoryLabel(entry.category)}</dd>
            </div>
            <div className="address-detail-row">
              <dt>メモ</dt>
              <dd>{entry.memo?.trim() ? entry.memo : '—'}</dd>
            </div>
          </dl>
        </section>

        <section className="address-detail-section">
          <h2 className="address-detail-section-title">送り主</h2>
          <dl className="address-detail-grid">
            <div className="address-detail-row">
              <dt>紐付け</dt>
              <dd>{entry.addressEntryId ? 'あり' : 'なし'}</dd>
            </div>
            {entry.addressEntryId ? (
              <>
                <div className="address-detail-row">
                  <dt>住所録</dt>
                  <dd>
                    {entry.addressEntryDisplayName ?? '—'}
                    {entry.addressEntryAddressLine ? ` / ${entry.addressEntryAddressLine}` : ''}
                    {entry.addressEntryArchived ? '（アーカイブ済みの宛名）' : ''}
                  </dd>
                </div>
                <div className="address-detail-row">
                  <dt>詳細</dt>
                  <dd>
                    <button
                      type="button"
                      className="link-button"
                      onClick={() => navigate(`/addresses/${entry.addressEntryId}`)}
                    >
                      住所録詳細へ
                    </button>
                  </dd>
                </div>
              </>
            ) : (
              <div className="address-detail-row">
                <dt>表示名</dt>
                <dd>{entry.senderDisplayName ?? '—'}</dd>
              </div>
            )}
          </dl>
        </section>

        <section className="address-detail-section">
          <h2 className="address-detail-section-title">システム情報</h2>
          <dl className="address-detail-grid">
            <div className="address-detail-row">
              <dt>ID</dt>
              <dd>{entry.id}</dd>
            </div>
            <div className="address-detail-row">
              <dt>作成日時</dt>
              <dd>{formatDateTime(entry.createdAt)}</dd>
            </div>
            <div className="address-detail-row">
              <dt>最終更新日時</dt>
              <dd>{formatDateTime(entry.updatedAt)}</dd>
            </div>
          </dl>
        </section>
      </main>
    </div>
  )
}
