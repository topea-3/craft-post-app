import { useCallback } from 'react'
import { useAddressEntryForm } from './useAddressEntryForm'
import { AddressEntryForm } from './AddressEntryForm'

export function AddressEntryCreatePage() {
  const handleSuccess = useCallback(() => {
    // v1: 作成後は一覧（ADDR001）想定だが、まだ一覧画面がないため
    // 当面は簡易的なアラートで通知のみ行う。
    // 一覧画面実装時にここで画面遷移を行う。
    // eslint-disable-next-line no-alert
    alert('住所録を登録しました。')
  }, [])

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
        '編集中の内容を破棄して一覧に戻ります。よろしいですか？（一覧画面は今後実装予定です）',
      )
      if (!confirmed) return
    }
    // 一覧画面未実装のため、現状は単にリロードして初期状態に戻す。
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

