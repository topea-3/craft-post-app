import { useEffect, useState, useCallback, useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { AddressEntryForm } from './AddressEntryForm'
import {
  type AddressEntryDto,
  type AddressEntryFormValues,
  type AddressEntryDetail,
  fromAddressEntryDtoToDetail,
  createInitialAddressEntryFormValues,
} from './types'
import { useAddressEntryEditForm } from './useAddressEntryForm'
import { formatDisplayName } from './types'
import { ADDRESS_OPERATION_ERROR_MESSAGE } from './messages'
import { FormSection } from '../../components/form/FormSection'
import { SelectField } from '../../components/form/SelectField'
import type { SenderEntryDto } from '../sender/types'
import { formatSenderDisplayName, fromSenderEntryDto } from '../sender/types'

/** `list_sender_entries` の limit 上限（lib.rs の MAX_PAGE_LIMIT）に合わせ、複数回呼び出して全件取得する */
const SENDER_LIST_PAGE_SIZE = 200

async function fetchAllActiveSenderEntryDtos(): Promise<SenderEntryDto[]> {
  const all: SenderEntryDto[] = []
  let offset = 0
  for (;;) {
    const page = await invoke<SenderEntryDto[]>('list_sender_entries', {
      limit: SENDER_LIST_PAGE_SIZE,
      offset,
    })
    all.push(...page)
    if (page.length < SENDER_LIST_PAGE_SIZE) {
      break
    }
    offset += SENDER_LIST_PAGE_SIZE
  }
  return all
}

export function AddressEntryEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()

  const [detail, setDetail] = useState<AddressEntryDetail | null>(null)
  const [initialValues, setInitialValues] = useState<AddressEntryFormValues | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [senderOptions, setSenderOptions] = useState<{ value: string; label: string }[]>([
    { value: '', label: '未設定' },
  ])
  const [linkedSenderId, setLinkedSenderId] = useState<string>('')

  const handleSuccess = useCallback(() => {
    if (!id) {
      navigate('/addresses')
      return
    }
    alert('住所録を更新しました。')
    navigate(`/addresses/${id}`)
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
        const dto = await invoke<AddressEntryDto>('get_address_entry', { id })
        if (cancelled) return

        const d = fromAddressEntryDtoToDetail(dto)
        setDetail(d)
        setInitialValues({
          primaryName: d.primaryName,
          coRecipients: d.coRecipients,
          honorific: d.honorific,
          address: d.address,
          memo: d.memo,
        })
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

  useEffect(() => {
    let cancelled = false
    const fetchSenders = async () => {
      try {
        const dtos = await fetchAllActiveSenderEntryDtos()
        if (cancelled) return
        const items = dtos.map(fromSenderEntryDto)
        const opts = [
          { value: '', label: '未設定' },
          ...items.map((s) => ({
            value: s.id,
            label: `${s.label}（${formatSenderDisplayName(s.primaryName, s.coRecipients) || '—'}）`,
          })),
        ]
        setSenderOptions(opts)
      } catch (e) {
        if (cancelled) return
        console.error('Failed to fetch sender entries:', e)
        setSenderOptions([{ value: '', label: '未設定' }])
      }
    }
    fetchSenders()
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    if (!id) return
    let cancelled = false
    const fetchLinkedSender = async () => {
      try {
        const senderId = await invoke<string | null>('get_sender_id_by_address_entry_id', {
          addressEntryId: id,
        })
        if (cancelled) return
        setLinkedSenderId(senderId ?? '')
      } catch (e) {
        if (cancelled) return
        console.error('Failed to fetch linked sender:', e)
        setLinkedSenderId('')
      }
    }
    fetchLinkedSender()
    return () => {
      cancelled = true
    }
  }, [id])

  const {
    values,
    errors,
    isSubmitting,
    isDirty,
    updatePrimaryName,
    updateAddress,
    addCoRecipient,
    updateCoRecipient,
    removeCoRecipient,
    updateHonorific,
    updateMemo,
    submit,
  } = useAddressEntryEditForm(
    id ?? '',
    useMemo(
      () => initialValues ?? createInitialAddressEntryFormValues(),
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
      navigate(`/addresses/${id}`)
    } else {
      navigate('/addresses')
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
        <p className="address-edit-error">住所録エントリの取得に失敗しました: {error}</p>
        <button
          type="button"
          onClick={() => {
            navigate('/addresses')
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
        <p>住所録エントリが見つかりませんでした。</p>
        <button
          type="button"
          onClick={() => {
            navigate('/addresses')
          }}
        >
          一覧に戻る
        </button>
      </div>
    )
  }

  const displayName = formatDisplayName(detail.primaryName, detail.coRecipients)

  return (
    <div className="address-edit-container">
      <header className="address-edit-header">
        <div>
          <h1 className="address-edit-title">住所録編集</h1>
          <div className="address-edit-subtitle">
            <span className="address-edit-name">{displayName}</span>
          </div>
        </div>
        <div className="address-edit-header-actions">
          <button
            type="button"
            onClick={async () => {
              const confirmed = window.confirm(
                'この住所録エントリをアーカイブしますか？一覧からは非表示になりますが、データは保持されます。',
              )
              if (!confirmed) return
              try {
                await invoke('archive_address_entry', { id: detail.id })
                alert('住所録エントリをアーカイブしました。')
                navigate('/addresses')
              } catch (error) {
                console.error('Failed to archive address entry:', error)
                alert(ADDRESS_OPERATION_ERROR_MESSAGE)
              }
            }}
          >
            アーカイブ
          </button>
          <button
            type="button"
            onClick={handleCancel}
          >
            戻る
          </button>
        </div>
      </header>

      <main className="address-edit-main">
        <AddressEntryForm
          mode="edit"
          values={values}
          errors={errors}
          isSubmitting={isSubmitting}
          onSubmit={submit}
          onCancel={handleCancel}
          onChangePrimaryName={updatePrimaryName}
          onChangeAddress={updateAddress}
          onAddCoRecipient={addCoRecipient}
          onChangeCoRecipient={updateCoRecipient}
          onRemoveCoRecipient={removeCoRecipient}
          onChangeHonorific={updateHonorific}
          onChangeMemo={updateMemo}
        />

        <FormSection title="差出人の紐づけ">
          <SelectField
            label="差出人"
            value={linkedSenderId}
            options={senderOptions}
            onChange={async (value) => {
              if (!id) return
              if (value === linkedSenderId) return

              // 付け替え時のみ確認（既存がある & 新しい値がある）
              if (linkedSenderId && value) {
                const confirmed = window.confirm(
                  'この宛名はすでに別の差出人に紐づいています。置き換えますか？',
                )
                if (!confirmed) return
              }

              try {
                await invoke('set_sender_for_address_entry', {
                  addressEntryId: id,
                  senderId: value.trim() === '' ? null : value,
                })
                setLinkedSenderId(value)
              } catch (e) {
                console.error('Failed to set sender for address:', e)
                alert(ADDRESS_OPERATION_ERROR_MESSAGE)
              }
            }}
            helperText="この宛名に印字する差出人を設定できます。"
          />
        </FormSection>
      </main>
    </div>
  )
}

