import { useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { SenderEntryForm } from './SenderEntryForm'
import { useSenderEntryForm } from './useSenderEntryForm'

export function SenderEntryCreatePage() {
  const navigate = useNavigate()

  const handleSuccess = useCallback(() => {
    alert('差出人を登録しました。')
    navigate('/senders')
  }, [navigate])

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
  } = useSenderEntryForm(handleSuccess)

  const handleCancel = () => {
    if (isDirty) {
      const confirmed = window.confirm(
        '編集中の内容を破棄して一覧に戻ります。よろしいですか？',
      )
      if (!confirmed) return
    }
    navigate('/senders')
  }

  return (
    <div className="sender-create-container">
      <header className="sender-create-header">
        <h1 className="sender-create-title">差出人登録（新規作成）</h1>
        <button type="button" className="link-button" onClick={handleCancel}>
          キャンセル
        </button>
      </header>
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
    </div>
  )
}

