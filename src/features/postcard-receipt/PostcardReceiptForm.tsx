import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { AddressEntrySelectDialog } from '../sender/AddressEntrySelectDialog'
import type { AddressEntryDto } from '../address/types'
import { fromAddressEntryDto } from '../address/types'
import type { UsePostcardReceiptFormResult } from './usePostcardReceiptForm'
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

  const handleSelectAddress = async (addressEntryId: string) => {
    try {
      const dto = await invoke<AddressEntryDto>('get_address_entry', { id: addressEntryId })
      const item = fromAddressEntryDto(dto)
      const displayName = formatAddressEntryLabel(
        item.primaryName,
        item.coRecipients,
        item.honorific,
      )
      setAddressEntry(addressEntryId, displayName)
    } catch (error) {
      console.error('Failed to load selected address entry:', error)
      setAddressEntry(addressEntryId, '選択済みの宛名')
    }
    setAddressDialogOpen(false)
  }

  return (
    <form className="address-form" onSubmit={handleSubmit}>
      <section className="address-form-section">
        <h2 className="address-form-section-title">受取情報</h2>
        <label className="address-form-field">
          <span>受取日</span>
          <input
            type="date"
            value={values.receivedAt}
            onChange={(e) => updateReceivedAt(e.target.value)}
          />
          {errors.receivedAt ? <span className="address-form-error">{errors.receivedAt}</span> : null}
        </label>

        <label className="address-form-field">
          <span>種別</span>
          <select
            value={values.category}
            onChange={(e) => updateCategory(e.target.value as typeof values.category)}
          >
            {POSTCARD_RECEIPT_CATEGORY_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          {errors.category ? <span className="address-form-error">{errors.category}</span> : null}
        </label>

        <label className="address-form-field">
          <span>メモ</span>
          <textarea value={values.memo} onChange={(e) => updateMemo(e.target.value)} rows={4} />
          {errors.memo ? <span className="address-form-error">{errors.memo}</span> : null}
        </label>
      </section>

      <section className="address-form-section">
        <h2 className="address-form-section-title">送り主</h2>
        <fieldset className="address-form-field">
          <legend>紐付け方法</legend>
          <label>
            <input
              type="radio"
              name="linkMode"
              checked={values.linkMode === 'address'}
              onChange={() => setLinkMode('address')}
            />
            住所録から選ぶ
          </label>
          <label>
            <input
              type="radio"
              name="linkMode"
              checked={values.linkMode === 'displayName'}
              onChange={() => setLinkMode('displayName')}
            />
            表示名のみ
          </label>
        </fieldset>

        {values.linkMode === 'address' ? (
          <div className="address-form-field">
            <span>住所録</span>
            <div>
              <button type="button" onClick={() => setAddressDialogOpen(true)}>
                宛名を選択
              </button>
              {values.addressEntryDisplayName ? (
                <p>
                  選択中: {values.addressEntryDisplayName}
                  <button type="button" onClick={clearAddressEntry}>
                    クリア
                  </button>
                </p>
              ) : null}
            </div>
            {errors.addressEntryId ? (
              <span className="address-form-error">{errors.addressEntryId}</span>
            ) : null}
          </div>
        ) : (
          <label className="address-form-field">
            <span>表示名</span>
            <input
              type="text"
              value={values.senderDisplayName}
              onChange={(e) => updateSenderDisplayName(e.target.value)}
            />
            {errors.senderDisplayName ? (
              <span className="address-form-error">{errors.senderDisplayName}</span>
            ) : null}
          </label>
        )}
      </section>

      {errors.form ? <p className="address-form-error">{errors.form}</p> : null}

      <div className="address-form-actions">
        <button type="button" className="secondary" onClick={onCancel}>
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
