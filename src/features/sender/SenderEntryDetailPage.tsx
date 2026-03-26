import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate, useParams } from 'react-router-dom'
import { SENDER_OPERATION_ERROR_MESSAGE } from './messages'
import type { SenderEntryDetail, SenderEntryDto } from './types'
import {
  formatSenderDisplayName,
  formatSenderKanaDisplayName,
  fromSenderEntryDtoToDetail,
} from './types'
import type { AddressEntryDto, AddressEntryListItem } from '../address/types'
import {
  fromAddressEntryDto,
  formatAddressSingleLine,
  formatDateTime,
  formatDisplayName,
  formatPostalCode,
} from '../address/types'

export function SenderEntryDetailPage() {
  const navigate = useNavigate()
  const { id } = useParams<{ id: string }>()
  const [entry, setEntry] = useState<SenderEntryDetail | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [linkedAddresses, setLinkedAddresses] = useState<AddressEntryListItem[]>([])

  useEffect(() => {
    if (!id) {
      setError('ID が指定されていません。')
      return
    }

    let cancelled = false
    const fetchDetail = async () => {
      setIsLoading(true)
      try {
        const dto = await invoke<SenderEntryDto>('get_sender_entry', { id })
        if (cancelled) return
        setEntry(fromSenderEntryDtoToDetail(dto))
        setError(null)
      } catch (e) {
        if (cancelled) return
        console.error('Failed to fetch sender entry detail:', e)
        setError(SENDER_OPERATION_ERROR_MESSAGE)
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

  useEffect(() => {
    if (!id) return
    let cancelled = false
    const fetchLinked = async () => {
      try {
        const dtos = await invoke<AddressEntryDto[]>('list_sender_linked_addresses', {
          senderId: id,
        })
        if (cancelled) return
        setLinkedAddresses(dtos.map(fromAddressEntryDto))
      } catch (e) {
        if (cancelled) return
        console.error('Failed to fetch sender linked addresses:', e)
        setLinkedAddresses([])
      }
    }
    fetchLinked()
    return () => {
      cancelled = true
    }
  }, [id])

  const handleArchive = async () => {
    if (!entry) return
    const confirmed = window.confirm(
      'この差出人をアーカイブしますか？一覧からは非表示になりますが、データは保持されます。',
    )
    if (!confirmed) return

    try {
      await invoke('archive_sender_entry', { id: entry.id })
      alert('差出人をアーカイブしました。')
      navigate('/senders')
    } catch (e) {
      console.error('Failed to archive sender entry:', e)
      alert(SENDER_OPERATION_ERROR_MESSAGE)
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
          onClick={() => {
            navigate('/senders')
          }}
        >
          一覧に戻る
        </button>
      </div>
    )
  }

  if (!entry) {
    return (
      <div className="address-detail-container">
        <p>差出人が見つかりませんでした。</p>
        <button
          type="button"
          onClick={() => {
            navigate('/senders')
          }}
        >
          一覧に戻る
        </button>
      </div>
    )
  }

  const displayName = formatSenderDisplayName(entry.primaryName, entry.coRecipients)
  const displayKanaName = formatSenderKanaDisplayName(entry.primaryName, entry.coRecipients)
  const primaryDisplayName =
    `${entry.primaryName.last.trim()} ${entry.primaryName.first.trim()}`.trim() || '—'
  const postalCode = formatPostalCode(entry.postalCode)
  const addressLine = formatAddressSingleLine(entry.address)
  const createdAt = formatDateTime(entry.createdAt)
  const updatedAt = formatDateTime(entry.updatedAt)

  return (
    <div className="address-detail-container">
      <header className="address-detail-header">
        <div>
          <h1 className="address-detail-title">差出人詳細</h1>
          <div className="address-detail-subtitle">
            <span className="address-detail-name">{displayName || '—'}</span>
          </div>
          {entry.archived && <span className="address-detail-badge">アーカイブ済み</span>}
        </div>
        <div className="address-detail-header-actions">
          <button
            type="button"
            className="primary"
            onClick={() => {
              navigate(`/senders/${entry.id}/edit`)
            }}
          >
            編集
          </button>
          <button type="button" onClick={handleArchive}>
            アーカイブ
          </button>
          <button
            type="button"
            onClick={() => {
              navigate('/senders')
            }}
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
              <dt>ラベル</dt>
              <dd>{entry.label}</dd>
            </div>
            <div className="address-detail-row">
              <dt>差出人（表示名）</dt>
              <dd>{displayName || '—'}</dd>
            </div>
            <div className="address-detail-row">
              <dt>差出人（カナ表示）</dt>
              <dd>{displayKanaName || '—'}</dd>
            </div>
            <div className="address-detail-row">
              <dt>主たる氏名</dt>
              <dd>{primaryDisplayName}</dd>
            </div>
            <div className="address-detail-row">
              <dt>連名</dt>
              <dd>
                {entry.coRecipients.length === 0
                  ? '—'
                  : entry.coRecipients.map((co, index) => {
                      const name = `${co.last ?? ''} ${co.first ?? ''}`.trim()
                      const display = name || '—'
                      return <div key={index}>{display}</div>
                    })}
              </dd>
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
          <h2 className="address-detail-section-title">電話番号</h2>
          <div className="address-detail-memo">{entry.phoneNumber || '未設定'}</div>
        </section>

        <section className="address-detail-section">
          <h2 className="address-detail-section-title">紐づき宛名</h2>
          {linkedAddresses.length === 0 ? (
            <div className="address-detail-memo">紐づき宛名はありません。</div>
          ) : (
            <table className="address-list-table" aria-label="紐づき宛名一覧テーブル">
              <thead>
                <tr>
                  <th scope="col">宛名</th>
                  <th scope="col">郵便番号</th>
                  <th scope="col">住所</th>
                </tr>
              </thead>
              <tbody>
                {linkedAddresses.map((a) => {
                  const displayName = formatDisplayName(a.primaryName, a.coRecipients)
                  const postal = formatPostalCode(a.postalCode)
                  const address = formatAddressSingleLine(a.address)
                  return (
                    <tr
                      key={a.id}
                      className="address-list-row"
                      onClick={() => {
                        navigate(`/addresses/${a.id}`)
                      }}
                    >
                      <td>
                        <span className="address-list-name">{displayName}</span>
                        <span className="address-list-honorific">{a.honorific}</span>
                      </td>
                      <td className="address-list-postal">{postal || a.postalCode}</td>
                      <td className="address-list-address" title={address}>
                        {address}
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          )}
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
              <dt>状態</dt>
              <dd>{entry.archived ? 'アーカイブ済み' : '有効'}</dd>
            </div>
          </dl>
        </section>
      </main>
    </div>
  )
}

