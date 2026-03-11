import { useCallback } from 'react'
import { useAddressEntryForm } from './useAddressEntryForm'
import { AddressEntryForm } from './AddressEntryForm'

export type AddressEntryCreatePageProps = {
  onCreated?: () => void
  onCancel?: () => void
}

export function AddressEntryCreatePage({ onCreated, onCancel }: AddressEntryCreatePageProps) {
  const handleSuccess = useCallback(() => {
    if (onCreated) {
      onCreated()
      return
    }
    // eslint-disable-next-line no-alert
    alert('住所録を登録しました。')
  }, [onCreated])

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
  } = useAddressEntryForm(handleSuccess)

  const handleCancel = () => {
    if (isDirty) {
      // eslint-disable-next-line no-alert
      const confirmed = window.confirm(
        '編集中の内容を破棄して一覧に戻ります。よろしいですか？',
      )
      if (!confirmed) return
    }
    if (onCancel) {
      onCancel()
      return
    }
    window.location.reload()
  }

  return (
    <AddressEntryForm
      mode="create"
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
  )
}

