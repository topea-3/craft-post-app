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

export function AddressEntryEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()

  const [detail, setDetail] = useState<AddressEntryDetail | null>(null)
  const [initialValues, setInitialValues] = useState<AddressEntryFormValues | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSuccess = useCallback(() => {
    if (!id) {
      navigate('/addresses')
      return
    }
    // eslint-disable-next-line no-alert
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
      } catch (e) {
        if (cancelled) return
        setError(String(e))
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
      // eslint-disable-next-line no-alert
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
                // eslint-disable-next-line no-alert
                alert('住所録エントリをアーカイブしました。')
                navigate('/addresses')
              } catch (e) {
                // eslint-disable-next-line no-alert
                alert(String(e))
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
      </main>
    </div>
  )
}

