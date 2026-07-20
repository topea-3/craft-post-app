import { useState } from 'react'
import { AddressEntrySelectDialog } from '../sender/AddressEntrySelectDialog'
import type { AddressEntryListItem } from '../address/types'
import {
  POSTCARD_RECEIPT_FIELD_IDS,
  type UsePostcardReceiptFormResult,
} from './usePostcardReceiptForm'
import { formatAddressEntryLabel, POSTCARD_RECEIPT_CATEGORY_OPTIONS } from './types'

type Props = {
  form: UsePostcardReceiptFormResult
  onCancel: () => void
  submitLabel?: string
}

export function PostcardReceiptForm({ form, onCancel, submitLabel = '保存' }: Props) {
  const {
    values,
    errors,
    isSubmitting,
    setLinkMode,
    setAddressEntry,
    clearAddressEntry,
    updateReceivedAt,
    updateCategory,
    updateSenderDisplayName,
    updateMemo,
    submit,
  } = form

  const [isAddressDialogOpen, setAddressDialogOpen] = useState(false)

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    await submit()
  }

  const handleSelectAddress = (item: AddressEntryListItem) => {
    const displayName = formatAddressEntryLabel(
      item.primaryName,
      item.coRecipients,
      item.honorific,
    )
    setAddressEntry(item.id, displayName)
    setAddressDialogOpen(false)
  }

  return (
    <form className="address-form" onSubmit={handleSubmit} noValidate>
      <section className="address-form-section">
        <h2 className="address-form-section-title">受取情報</h2>
        <label className="address-form-field" htmlFor={POSTCARD_RECEIPT_FIELD_IDS.receivedAt}>
          <span>受取日</span>
          <input
            id={POSTCARD_RECEIPT_FIELD_IDS.receivedAt}
            type="date"
            value={values.receivedAt}
            onChange={(e) => updateReceivedAt(e.target.value)}
            disabled={isSubmitting}
            aria-invalid={Boolean(errors.receivedAt)}
            aria-describedby={errors.receivedAt ? `${POSTCARD_RECEIPT_FIELD_IDS.receivedAt}-error` : undefined}
          />
          {errors.receivedAt ? (
            <span
              id={`${POSTCARD_RECEIPT_FIELD_IDS.receivedAt}-error`}
              className="address-form-error"
              role="alert"
            >
              {errors.receivedAt}
            </span>
          ) : null}
        </label>

        <label className="address-form-field" htmlFor={POSTCARD_RECEIPT_FIELD_IDS.category}>
          <span>種別</span>
          <select
            id={POSTCARD_RECEIPT_FIELD_IDS.category}
            value={values.category}
            onChange={(e) => updateCategory(e.target.value as typeof values.category)}
            disabled={isSubmitting}
            aria-invalid={Boolean(errors.category)}
            aria-describedby={errors.category ? `${POSTCARD_RECEIPT_FIELD_IDS.category}-error` : undefined}
          >
            {POSTCARD_RECEIPT_CATEGORY_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          {errors.category ? (
            <span
              id={`${POSTCARD_RECEIPT_FIELD_IDS.category}-error`}
              className="address-form-error"
              role="alert"
            >
              {errors.category}
            </span>
          ) : null}
        </label>

        <label className="address-form-field" htmlFor={POSTCARD_RECEIPT_FIELD_IDS.memo}>
          <span>メモ</span>
          <textarea
            id={POSTCARD_RECEIPT_FIELD_IDS.memo}
            value={values.memo}
            onChange={(e) => updateMemo(e.target.value)}
            rows={4}
            disabled={isSubmitting}
            aria-invalid={Boolean(errors.memo)}
            aria-describedby={errors.memo ? `${POSTCARD_RECEIPT_FIELD_IDS.memo}-error` : undefined}
          />
          {errors.memo ? (
            <span
              id={`${POSTCARD_RECEIPT_FIELD_IDS.memo}-error`}
              className="address-form-error"
              role="alert"
            >
              {errors.memo}
            </span>
          ) : null}
        </label>
      </section>

      <section className="address-form-section">
        <h2 className="address-form-section-title">送り主</h2>
        <fieldset className="address-form-field" disabled={isSubmitting}>
          <legend>紐付け方法</legend>
          <label>
            <input
              type="radio"
              name="linkMode"
              checked={values.linkMode === 'address'}
              onChange={() => setLinkMode('address')}
              disabled={isSubmitting}
            />
            住所録から選ぶ
          </label>
          <label>
            <input
              type="radio"
              name="linkMode"
              checked={values.linkMode === 'displayName'}
              onChange={() => setLinkMode('displayName')}
              disabled={isSubmitting}
            />
            表示名のみ
          </label>
        </fieldset>

        {values.linkMode === 'address' ? (
          <div className="address-form-field">
            <span id={`${POSTCARD_RECEIPT_FIELD_IDS.addressEntryId}-label`}>住所録</span>
            <div>
              <button
                id={POSTCARD_RECEIPT_FIELD_IDS.addressEntryId}
                type="button"
                onClick={() => setAddressDialogOpen(true)}
                disabled={isSubmitting}
                aria-invalid={Boolean(errors.addressEntryId)}
                aria-describedby={
                  errors.addressEntryId ? `${POSTCARD_RECEIPT_FIELD_IDS.addressEntryId}-error` : undefined
                }
                aria-labelledby={`${POSTCARD_RECEIPT_FIELD_IDS.addressEntryId}-label`}
              >
                宛名を選択
              </button>
              {values.addressEntryDisplayName ? (
                <p>
                  選択中: {values.addressEntryDisplayName}
                  <button type="button" onClick={clearAddressEntry} disabled={isSubmitting}>
                    クリア
                  </button>
                </p>
              ) : null}
            </div>
            {errors.addressEntryId ? (
              <span
                id={`${POSTCARD_RECEIPT_FIELD_IDS.addressEntryId}-error`}
                className="address-form-error"
                role="alert"
              >
                {errors.addressEntryId}
              </span>
            ) : null}
          </div>
        ) : (
          <label className="address-form-field" htmlFor={POSTCARD_RECEIPT_FIELD_IDS.senderDisplayName}>
            <span>表示名</span>
            <input
              id={POSTCARD_RECEIPT_FIELD_IDS.senderDisplayName}
              type="text"
              value={values.senderDisplayName}
              onChange={(e) => updateSenderDisplayName(e.target.value)}
              disabled={isSubmitting}
              aria-invalid={Boolean(errors.senderDisplayName)}
              aria-describedby={
                errors.senderDisplayName
                  ? `${POSTCARD_RECEIPT_FIELD_IDS.senderDisplayName}-error`
                  : undefined
              }
            />
            {errors.senderDisplayName ? (
              <span
                id={`${POSTCARD_RECEIPT_FIELD_IDS.senderDisplayName}-error`}
                className="address-form-error"
                role="alert"
              >
                {errors.senderDisplayName}
              </span>
            ) : null}
          </label>
        )}
      </section>

      {errors.form ? (
        <p
          id={POSTCARD_RECEIPT_FIELD_IDS.form}
          className="address-form-error"
          role="alert"
          tabIndex={-1}
        >
          {errors.form}
        </p>
      ) : null}

      <div className="address-form-actions">
        <button type="button" className="secondary" onClick={onCancel} disabled={isSubmitting}>
          キャンセル
        </button>
        <button type="submit" disabled={isSubmitting}>
          {isSubmitting ? '保存中…' : submitLabel}
        </button>
      </div>

      {isAddressDialogOpen ? (
        <AddressEntrySelectDialog
          isOpen={isAddressDialogOpen}
          onClose={() => setAddressDialogOpen(false)}
          onSelect={handleSelectAddress}
        />
      ) : null}
    </form>
  )
}
