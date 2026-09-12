import type React from 'react'
import { FormSection } from '../../components/form/FormSection'
import { TextField } from '../../components/form/TextField'
import { PrimaryNameForm } from '../address/PrimaryNameForm'
import { CoRecipientsForm } from '../address/CoRecipientsForm'
import { AddressForm } from '../address/AddressForm'
import type { SenderEntryFormValues } from './types'
import { formatSenderDisplayName, SENDER_LABEL_MAX_LENGTH } from './types'
import type { SenderEntryFormErrors } from './useSenderEntryForm'

type Props = {
  values: SenderEntryFormValues
  errors: SenderEntryFormErrors
  isSubmitting: boolean
  onSubmit: () => void
  onCancel: () => void
  onChangeLabel: (value: string) => void
  onChangePrimaryName: (patch: Partial<SenderEntryFormValues['primaryName']>) => void
  onChangeAddress: (patch: Partial<SenderEntryFormValues['address']>) => void
  onAddCoRecipient: () => void
  onChangeCoRecipient: (
    index: number,
    patch: Partial<SenderEntryFormValues['coRecipients'][number]>,
  ) => void
  onRemoveCoRecipient: (index: number) => void
  onChangePhoneNumber: (value: string) => void
}

export function SenderEntryForm({
  values,
  errors,
  isSubmitting,
  onSubmit,
  onCancel,
  onChangeLabel,
  onChangePrimaryName,
  onChangeAddress,
  onAddCoRecipient,
  onChangeCoRecipient,
  onRemoveCoRecipient,
  onChangePhoneNumber,
}: Props) {
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSubmit()
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLFormElement>) => {
    if (e.key === 'Enter' && e.target instanceof HTMLInputElement) {
      e.preventDefault()
    }
  }

  const displayName = formatSenderDisplayName(values.primaryName, values.coRecipients)

  return (
    <form className="address-entry-form" onSubmit={handleSubmit} onKeyDown={handleKeyDown}>
      {errors.form ? <div className="form-error-banner">{errors.form}</div> : null}

      <FormSection title="差出人情報">
        <TextField
          label="ラベル"
          value={values.label}
          required
          maxLength={SENDER_LABEL_MAX_LENGTH}
          error={errors.label}
          onChange={onChangeLabel}
        />
      </FormSection>

      <FormSection title="氏名（主たる氏名・連名）">
        <PrimaryNameForm
          value={values.primaryName}
          errors={errors.primaryName}
          onChange={onChangePrimaryName}
        />
        <CoRecipientsForm
          values={values.coRecipients}
          errors={errors.coRecipients}
          onAdd={onAddCoRecipient}
          onChangeRow={onChangeCoRecipient}
          onRemove={onRemoveCoRecipient}
        />
      </FormSection>

      <FormSection title="差出人の表示名プレビュー">
        <p className="sender-display-preview">{displayName || '—'}</p>
      </FormSection>

      <FormSection title="住所情報">
        <AddressForm value={values.address} errors={errors.address} onChange={onChangeAddress} />
      </FormSection>

      <FormSection title="電話番号（任意）">
        <TextField
          label="電話番号"
          value={values.phoneNumber ?? ''}
          error={errors.phoneNumber}
          onChange={onChangePhoneNumber}
        />
      </FormSection>

      <div className="form-footer">
        <button type="button" className="secondary" onClick={onCancel}>
          キャンセル
        </button>
        <button type="submit" className="primary" disabled={isSubmitting}>
          保存
        </button>
      </div>
    </form>
  )
}

