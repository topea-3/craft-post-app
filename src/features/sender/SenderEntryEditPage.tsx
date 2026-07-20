import { useCallback, useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate, useParams } from 'react-router-dom'
import { SenderEntryForm } from './SenderEntryForm'
import type { SenderEntryDetail, SenderEntryDto, SenderEntryFormValues } from './types'
import { createInitialSenderEntryFormValues, formatSenderDisplayName, fromSenderEntryDtoToDetail } from './types'
import { useSenderEntryEditForm } from './useSenderEntryForm'
import { SENDER_OPERATION_ERROR_MESSAGE } from './messages'
import type { AddressEntryDto, AddressEntryListItem } from '../address/types'
import {
  fromAddressEntryDto,
  formatAddressSingleLine,
  formatDisplayName,
  formatPostalCode,
} from '../address/types'
import { AddressEntrySelectDialog } from './AddressEntrySelectDialog'

export function SenderEntryEditPage() {
  const navigate = useNavigate()
  const { id } = useParams<{ id: string }>()
  const [detail, setDetail] = useState<SenderEntryDetail | null>(null)
  const [initialValues, setInitialValues] = useState<SenderEntryFormValues | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [linkedAddresses, setLinkedAddresses] = useState<AddressEntryListItem[]>([])
  const [isSelectDialogOpen, setIsSelectDialogOpen] = useState(false)

  const handleSuccess = useCallback(() => {
    if (!id) {
      navigate('/senders')
      return
    }
    alert('差出人を更新しました。')
    navigate(`/senders/${id}`)
  }, [id, navigate])

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
        const d = fromSenderEntryDtoToDetail(dto)
        setDetail(d)
        setInitialValues({
          label: d.label,
          primaryName: d.primaryName,
          coRecipients: d.coRecipients,
          address: d.address,
          phoneNumber: d.phoneNumber,
        })
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

  const reloadLinked = useCallback(async () => {
    if (!id) return
    try {
      const dtos = await invoke<AddressEntryDto[]>('list_sender_linked_addresses', {
        senderId: id,
      })
      setLinkedAddresses(dtos.map(fromAddressEntryDto))
    } catch (e) {
      console.error('Failed to reload sender linked addresses:', e)
      setLinkedAddresses([])
    }
  }, [id])

  const {
    values,
    errors,
    isSubmitting,
    isDirty,
    updateLabel,
    updatePrimaryName,
    updateAddress,
    addCoRecipient,
    updateCoRecipient,
    removeCoRecipient,
    updatePhoneNumber,
    submit,
  } = useSenderEntryEditForm(
    id ?? '',
    useMemo(
      () => initialValues ?? createInitialSenderEntryFormValues(),
      [initialValues],
    ),
    handleSuccess,
  )

  const handleCancel = () => {
    if (isDirty) {
      const confirmed = window.confirm(
        '編集中の内容を破棄して詳細画面に戻ります。よろしいですか？',
      )
      if (!confirmed) return
    }
    if (id) {
      navigate(`/senders/${id}`)
    } else {
      navigate('/senders')
    }
  }

  const handleArchive = async () => {
    if (!detail) return
    const confirmed = window.confirm(
      'この差出人をアーカイブしますか？一覧からは非表示になりますが、データは保持されます。',
    )
    if (!confirmed) return
    try {
      await invoke('archive_sender_entry', { id: detail.id })
      alert('差出人をアーカイブしました。')
      navigate('/senders')
    } catch (e) {
      console.error('Failed to archive sender entry:', e)
      alert(SENDER_OPERATION_ERROR_MESSAGE)
    }
  }

  const handleUnlink = async (addressEntryId: string) => {
    if (!id) return
    const nextIds = linkedAddresses
      .filter((a) => a.id !== addressEntryId)
      .map((a) => a.id)
    try {
      await invoke('update_sender_entry_links', {
        senderId: id,
        addressEntryIds: nextIds,
      })
      setLinkedAddresses((prev) => prev.filter((a) => a.id !== addressEntryId))
    } catch (e) {
      console.error('Failed to unlink sender address:', e)
      alert(SENDER_OPERATION_ERROR_MESSAGE)
    }
  }

  const handleAddLink = async (item: AddressEntryListItem): Promise<boolean> => {
    if (!id) return false

    const addressEntryId = item.id

    // 既にこの差出人に紐づいている場合は何もしない
    if (linkedAddresses.some((a) => a.id === addressEntryId)) {
      return false
    }

    try {
      const existingSenderId = await invoke<string | null>('get_sender_id_by_address_entry_id', {
        addressEntryId,
      })
      if (existingSenderId && existingSenderId !== id) {
        const confirmed = window.confirm(
          'この宛名はすでに別の差出人に紐づいています。置き換えますか？',
        )
        if (!confirmed) return false
      }

      const nextIds = [...linkedAddresses.map((a) => a.id), addressEntryId]
      await invoke('update_sender_entry_links', {
        senderId: id,
        addressEntryIds: nextIds,
      })
      await reloadLinked()
      return true
    } catch (e) {
      console.error('Failed to add sender link:', e)
      alert(SENDER_OPERATION_ERROR_MESSAGE)
      return false
    }
  }

  if (isLoading && !detail) {
    return (
      <div className="address-edit-container">
        <p>読み込み中です…</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="address-edit-container">
        <p className="address-edit-error">差出人の取得に失敗しました: {error}</p>
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

  if (!detail) {
    return (
      <div className="address-edit-container">
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

  const displayName = formatSenderDisplayName(detail.primaryName, detail.coRecipients)

  return (
    <div className="address-edit-container">
      <header className="address-edit-header">
        <div>
          <h1 className="address-edit-title">差出人編集</h1>
          <div className="address-edit-subtitle">
            <span className="address-edit-name">{displayName || '—'}</span>
          </div>
        </div>
        <div className="address-edit-header-actions">
          <button type="button" onClick={handleArchive}>
            アーカイブ
          </button>
          <button type="button" onClick={handleCancel}>
            戻る
          </button>
        </div>
      </header>

      <main className="address-edit-main">
        <SenderEntryForm
          values={values}
          errors={errors}
          isSubmitting={isSubmitting}
          onSubmit={submit}
          onCancel={handleCancel}
          onChangeLabel={updateLabel}
          onChangePrimaryName={updatePrimaryName}
          onChangeAddress={updateAddress}
          onAddCoRecipient={addCoRecipient}
          onChangeCoRecipient={updateCoRecipient}
          onRemoveCoRecipient={removeCoRecipient}
          onChangePhoneNumber={updatePhoneNumber}
        />

        <section className="form-section">
          <header className="form-section-header">
            <h2 className="form-section-title">紐づき宛名</h2>
            <p className="form-section-description">
              この差出人として印字する宛名（住所録）を管理します。
            </p>
          </header>
          <div className="form-section-body">
            {linkedAddresses.length === 0 ? (
              <p>紐づき宛名はありません。</p>
            ) : (
              <table className="address-list-table" aria-label="紐づき宛名一覧テーブル">
                <thead>
                  <tr>
                    <th scope="col">宛名</th>
                    <th scope="col">郵便番号</th>
                    <th scope="col">住所</th>
                    <th scope="col">操作</th>
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
                        <td className="address-list-actions">
                          <button
                            type="button"
                            onClick={(e) => {
                              e.stopPropagation()
                              handleUnlink(a.id)
                            }}
                          >
                            解除
                          </button>
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            )}

            <button
              type="button"
              onClick={() => {
                setIsSelectDialogOpen(true)
              }}
            >
              紐づけ追加
            </button>
          </div>
        </section>
      </main>

      <AddressEntrySelectDialog
        isOpen={isSelectDialogOpen}
        excludeIds={linkedAddresses.map((a) => a.id)}
        onClose={() => setIsSelectDialogOpen(false)}
        onSelect={handleAddLink}
      />
    </div>
  )
}

