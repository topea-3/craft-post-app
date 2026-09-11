import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { POSTCARD_SEND_OPERATION_ERROR_MESSAGE } from './messages'
import type { PostcardSendDetail, PostcardSendDto } from './types'
import {
  formatDateTime,
  formatSentOn,
  fromPostcardSendDtoToDetail,
  postcardTypeLabel,
  resolveAddressDisplayName,
  resolveAddressPostalAndLine,
  resolveSenderDisplayName,
  resolveSenderPostalAndLine,
  sourceLabel,
} from './types'

export function PostcardSendDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [detail, setDetail] = useState<PostcardSendDetail | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [deleting, setDeleting] = useState(false)
  const deletingRef = useRef(false)

  useEffect(() => {
    if (!id) return
    let cancelled = false
    ;(async () => {
      setLoading(true)
      try {
        const dto = await invoke<PostcardSendDto>('get_postcard_send', { id })
        if (cancelled) return
        setDetail(fromPostcardSendDtoToDetail(dto))
        setError(null)
      } catch (e) {
        console.error(e)
        if (!cancelled) setError(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)
      } finally {
        if (!cancelled) setLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [id])

  const handleDelete = () => {
    if (!id || deletingRef.current) return
    const confirmed = window.confirm('この送付履歴を削除しますか？一覧からは非表示になります。')
    if (!confirmed) return
    deletingRef.current = true
    setDeleting(true)
    ;(async () => {
      try {
        await invoke('delete_postcard_send', { id })
        navigate('/sends')
      } catch (e) {
        console.error(e)
        alert(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)
        setDeleting(false)
        deletingRef.current = false
      }
    })()
  }

  if (loading) return <p>読み込み中…</p>
  if (error || !detail) {
    return (
      <div className="address-form-container">
        <p className="address-form-error">{error ?? '送付履歴が見つかりません。'}</p>
        <Link to="/sends">一覧へ</Link>
      </div>
    )
  }

  const addressPostal = resolveAddressPostalAndLine(detail)
  const senderPostal = resolveSenderPostalAndLine(detail)
  const addressActive = detail.addressEntryArchived === false
  const addressArchived = detail.addressEntryArchived === true
  const senderActive = detail.senderEntryArchived === false
  const senderArchived = detail.senderEntryArchived === true

  return (
    <div className="address-form-container">
      <header className="address-form-header">
        <h1>送付履歴詳細</h1>
        <div>
          <Link to="/sends" className="link-button">
            一覧へ
          </Link>
          <button
            type="button"
            className="link-button"
            disabled={deleting}
            onClick={() => navigate(`/sends/${detail.id}/edit`)}
          >
            編集
          </button>
          <button type="button" className="link-button" disabled={deleting} onClick={handleDelete}>
            削除
          </button>
        </div>
      </header>

      <dl className="address-detail-list">
        <div>
          <dt>送付日</dt>
          <dd>{formatSentOn(detail.sentOn)}</dd>
        </div>
        <div>
          <dt>種別</dt>
          <dd>{postcardTypeLabel(detail.postcardType)}</dd>
        </div>
        <div>
          <dt>登録経路</dt>
          <dd>{sourceLabel(detail.source)}</dd>
        </div>
        <div>
          <dt>メモ</dt>
          <dd style={{ whiteSpace: 'pre-wrap' }}>{detail.memo?.trim() || '—'}</dd>
        </div>

        <div>
          <dt>宛名（送付時）</dt>
          <dd>
            <div>{resolveAddressDisplayName(detail)}</div>
            {addressPostal.postalCode ? <div>{addressPostal.postalCode}</div> : null}
            <div>{addressPostal.addressLine}</div>
            {addressActive ? (
              <div>
                <Link to={`/addresses/${detail.addressEntryId}`}>住所録を開く</Link>
              </div>
            ) : null}
            {addressArchived ? <div className="address-form-help">アーカイブ済み</div> : null}
          </dd>
        </div>

        <div>
          <dt>差出人（送付時）</dt>
          <dd>
            <div>{resolveSenderDisplayName(detail)}</div>
            {senderPostal.postalCode ? <div>{senderPostal.postalCode}</div> : null}
            <div>{senderPostal.addressLine}</div>
            {senderActive ? (
              <div>
                <Link to={`/senders/${detail.senderEntryId}`}>差出人を開く</Link>
              </div>
            ) : null}
            {senderArchived ? <div className="address-form-help">アーカイブ済み</div> : null}
          </dd>
        </div>

        <div>
          <dt>作成日時</dt>
          <dd>{formatDateTime(detail.createdAt)}</dd>
        </div>
        <div>
          <dt>更新日時</dt>
          <dd>{formatDateTime(detail.updatedAt)}</dd>
        </div>
      </dl>
    </div>
  )
}
