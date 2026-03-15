import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import type { AddressEntryDetail, AddressEntryDto } from './types'
import {
  formatAddressSingleLine,
  formatDateTime,
  formatDisplayName,
  formatPostalCode,
  fromAddressEntryDtoToDetail,
} from './types'
import { ADDRESS_OPERATION_ERROR_MESSAGE } from './messages'

export function AddressEntryDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [entry, setEntry] = useState<AddressEntryDetail | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!id) {
      setError('ID が指定されていません。')
      return
    }

    let cancelled = false

    const fetchDetail = async () => {
      setIsLoading(true)
      try {
        const dto = await invoke<AddressEntryDto>('get_address_entry', { id })
        if (cancelled) return
        setEntry(fromAddressEntryDtoToDetail(dto))
        setError(null)
      } catch (error) {
        if (cancelled) return
        console.error('Failed to fetch address entry detail:', error)
        setError(ADDRESS_OPERATION_ERROR_MESSAGE)
      } finally {
        if (!cancelled) {
          setIsLoading(false)
        }
      }
    }

    fetchDetail()

    return () => {
      cancelled = true
    }
  }, [id])

  const handleBackToList = () => {
    navigate('/addresses')
  }

  const handleEdit = () => {
    if (!id && !entry) {
      navigate('/addresses')
      return
    }
    const targetId = id ?? entry?.id
    if (!targetId) {
      navigate('/addresses')
      return
    }
    navigate(`/addresses/${targetId}/edit`)
  }

  const handleArchive = async () => {
    if (!entry) return
    const confirmed = window.confirm(
      'この住所録エントリをアーカイブしますか？一覧からは非表示になりますが、データは保持されます。',
    )
    if (!confirmed) return

    try {
      await invoke('archive_address_entry', { id: entry.id })
      setEntry({ ...entry, archived: true })
      alert('住所録エントリをアーカイブしました。')
    } catch (error) {
      console.error('Failed to archive address entry:', error)
      alert(ADDRESS_OPERATION_ERROR_MESSAGE)
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
        <button
          type="button"
          onClick={handleBackToList}
        >
          一覧に戻る
        </button>
      </div>
    )
  }

  if (!entry) {
    return (
      <div className="address-detail-container">
        <p>住所録エントリが見つかりませんでした。</p>
        <button
          type="button"
          onClick={handleBackToList}
        >
          一覧に戻る
        </button>
      </div>
    )
  }

  const displayName = formatDisplayName(entry.primaryName, entry.coRecipients)
  const primaryDisplayName =
    `${entry.primaryName.last.trim()} ${entry.primaryName.first.trim()}`.trim() ||
    '—'
  const postalCode = formatPostalCode(entry.postalCode)
  const addressLine = formatAddressSingleLine(entry.address)
  const createdAt = formatDateTime(entry.createdAt)
  const updatedAt = formatDateTime(entry.updatedAt)

  return (
    <div className="address-detail-container">
      <header className="address-detail-header">
        <div>
          <h1 className="address-detail-title">住所録詳細</h1>
          <div className="address-detail-subtitle">
            <span className="address-detail-name">{displayName}</span>
            {entry.coRecipients.length > 0 && (
              <span className="address-detail-subtext">
                {/* 連名の詳細は下部ブロックで表示 */}
              </span>
            )}
          </div>
          {entry.archived && (
            <span className="address-detail-badge">アーカイブ済み</span>
          )}
        </div>
        <div className="address-detail-header-actions">
          <button
            type="button"
            onClick={handleEdit}
          >
            編集
          </button>
          <button
            type="button"
            onClick={handleArchive}
          >
            アーカイブ
          </button>
          <button
            type="button"
            onClick={handleBackToList}
          >
            戻る
          </button>
        </div>
      </header>

      <main className="address-detail-main">
        <section className="address-detail-section">
          <h2 className="address-detail-section-title">基本情報</h2>
          <dl className="address-detail-grid">
            <div className="address-detail-row">
              <dt>主たる氏名</dt>
              <dd>{primaryDisplayName}</dd>
            </div>
            <div className="address-detail-row">
              <dt>主たる氏名（カナ）</dt>
              <dd>
                {[
                  entry.primaryName.kanaLast ?? '',
                  entry.primaryName.kanaFirst ?? '',
                ]
                  .join(' ')
                  .trim() || '—'}
              </dd>
            </div>
            <div className="address-detail-row">
              <dt>連名</dt>
              <dd>
                {entry.coRecipients.length === 0
                  ? '—'
                  : entry.coRecipients.map((co, index) => {
                      const name = `${co.last ?? ''} ${co.first ?? ''}`.trim()
                      const display = name || '—'
                      return (
                        <div key={index}>
                          {display}
                        </div>
                      )
                    })}
              </dd>
            </div>
            <div className="address-detail-row">
              <dt>敬称</dt>
              <dd>{entry.honorific}</dd>
            </div>
          </dl>
        </section>

        <section className="address-detail-section">
          <h2 className="address-detail-section-title">住所情報</h2>
          <dl className="address-detail-grid">
            <div className="address-detail-row">
              <dt>郵便番号</dt>
              <dd>{postalCode || entry.postalCode}</dd>
            </div>
            <div className="address-detail-row">
              <dt>都道府県</dt>
              <dd>{entry.address.prefecture}</dd>
            </div>
            <div className="address-detail-row">
              <dt>市区町村</dt>
              <dd>{entry.address.city}</dd>
            </div>
            <div className="address-detail-row">
              <dt>町名・番地</dt>
              <dd>{entry.address.street}</dd>
            </div>
            <div className="address-detail-row">
              <dt>建物名・部屋番号</dt>
              <dd>{entry.address.building || '—'}</dd>
            </div>
            <div className="address-detail-row">
              <dt>住所（1 行）</dt>
              <dd>{addressLine}</dd>
            </div>
          </dl>
        </section>

        <section className="address-detail-section">
          <h2 className="address-detail-section-title">メモ</h2>
          <div className="address-detail-memo">
            {entry.memo && entry.memo.trim()
              ? entry.memo.split('\n').map((line, index) => (
                  <span key={index}>
                    {line}
                    {index < entry.memo!.split('\n').length - 1 && <br />}
                  </span>
                ))
              : '—'}
          </div>
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
              <dd>{createdAt}</dd>
            </div>
            <div className="address-detail-row">
              <dt>最終更新日時</dt>
              <dd>{updatedAt}</dd>
            </div>
            <div className="address-detail-row">
              <dt>ステータス</dt>
              <dd>{entry.archived ? 'アーカイブ済み' : '有効'}</dd>
            </div>
          </dl>
        </section>
      </main>
    </div>
  )
}

