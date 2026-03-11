import { FormSection } from '../../components/form/FormSection'
import { PrimaryNameForm } from './PrimaryNameForm'
import { CoRecipientsForm } from './CoRecipientsForm'
import { HonorificSelect } from './HonorificSelect'
import { AddressForm } from './AddressForm'
import { MemoField } from './MemoField'
import type { AddressEntryFormValues } from './types'
import type { AddressEntryFormErrors } from './useAddressEntryForm'

type Props = {
  mode: 'create' | 'edit'
  values: AddressEntryFormValues
  errors: AddressEntryFormErrors
  isSubmitting: boolean
  onSubmit: () => void
  onCancel: () => void
  onChangePrimaryName: (patch: Partial<AddressEntryFormValues['primaryName']>) => void
  onChangeAddress: (patch: Partial<AddressEntryFormValues['address']>) => void
  onAddCoRecipient: () => void
  onChangeCoRecipient: (
    index: number,
    patch: Partial<AddressEntryFormValues['coRecipients'][number]>,
  ) => void
  onRemoveCoRecipient: (index: number) => void
  onChangeHonorific: (value: string) => void
  onChangeMemo: (value: string) => void
}

export function AddressEntryForm({
  mode,
  values,
  errors,
  isSubmitting,
  onSubmit,
  onCancel,
  onChangePrimaryName,
  onChangeAddress,
  onAddCoRecipient,
  onChangeCoRecipient,
  onRemoveCoRecipient,
  onChangeHonorific,
  onChangeMemo,
}: Props) {
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSubmit()
  }

  return (
    <form className="address-entry-form" onSubmit={handleSubmit}>
      {errors.form ? <div className="form-error-banner">{errors.form}</div> : null}

      <FormSection title="宛先情報（氏名・連名・敬称）">
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
        <HonorificSelect
          value={values.honorific}
          error={errors.honorific}
          onChange={onChangeHonorific}
        />
      </FormSection>

      <FormSection title="住所情報">
        <AddressForm value={values.address} errors={errors.address} onChange={onChangeAddress} />
      </FormSection>

      <FormSection title="メモ">
        <MemoField value={values.memo ?? ''} error={errors.memo} onChange={onChangeMemo} />
      </FormSection>

      <div className="form-footer">
        <button type="button" className="secondary" onClick={onCancel}>
          キャンセル
        </button>
        <button type="submit" className="primary" disabled={isSubmitting}>
          {mode === 'create' ? '保存' : '更新'}
        </button>
      </div>
    </form>
  )
}

