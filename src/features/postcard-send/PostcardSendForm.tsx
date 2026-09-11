import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { AddressEntrySelectDialog } from '../sender/AddressEntrySelectDialog'
import { SenderEntrySelectDialog } from '../sender/SenderEntrySelectDialog'
import type { AddressEntryListItem } from '../address/types'
import { formatDisplayName } from '../address/types'
import { POSTCARD_TYPE_OPTIONS } from './types'
import {
  POSTCARD_SEND_FIELD_IDS,
  type UsePostcardSendFormResult,
} from './usePostcardSendForm'

type Props = {
  form: UsePostcardSendFormResult
  onCancel: () => void
  submitLabel?: string
}

export function PostcardSendForm({ form, onCancel, submitLabel = '保存' }: Props) {
  const [addressDialogOpen, setAddressDialogOpen] = useState(false)
  const [senderDialogOpen, setSenderDialogOpen] = useState(false)
  const { values, errors, isSubmitting, allowIdentityEdit } = form

  return (
    <form
      className="address-form"
      onSubmit={(e) => {
        e.preventDefault()
        void form.submit()
      }}
    >
      {errors.form ? (
        <p
          id={POSTCARD_SEND_FIELD_IDS.form}
          className="address-form-error"
          role="alert"
          tabIndex={-1}
        >
          {errors.form}
        </p>
      ) : null}

      <fieldset className="address-form-section" disabled={isSubmitting}>
        <legend>送付情報</legend>
        <label className="address-form-label" htmlFor={POSTCARD_SEND_FIELD_IDS.sentOn}>
          送付日
          <input
            id={POSTCARD_SEND_FIELD_IDS.sentOn}
            type="date"
            value={values.sentOn}
            onChange={(e) => form.updateSentOn(e.target.value)}
            aria-invalid={Boolean(errors.sentOn)}
          />
        </label>
        {errors.sentOn ? <p className="address-form-error">{errors.sentOn}</p> : null}

        <label className="address-form-label" htmlFor={POSTCARD_SEND_FIELD_IDS.postcardType}>
          種別
          <select
            id={POSTCARD_SEND_FIELD_IDS.postcardType}
            value={values.postcardType}
            onChange={(e) =>
              form.updatePostcardType(e.target.value as 'nenga' | 'mochu')
            }
            aria-invalid={Boolean(errors.postcardType)}
          >
            {POSTCARD_TYPE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        {errors.postcardType ? <p className="address-form-error">{errors.postcardType}</p> : null}

        <label className="address-form-label" htmlFor={POSTCARD_SEND_FIELD_IDS.memo}>
          メモ
          <textarea
            id={POSTCARD_SEND_FIELD_IDS.memo}
            value={values.memo}
            onChange={(e) => form.updateMemo(e.target.value)}
            rows={4}
            aria-invalid={Boolean(errors.memo)}
          />
        </label>
        {errors.memo ? <p className="address-form-error">{errors.memo}</p> : null}
      </fieldset>

      <fieldset className="address-form-section" disabled={isSubmitting || !allowIdentityEdit}>
        <legend>宛名・差出人</legend>
        {allowIdentityEdit ? (
          <p className="address-form-help">
            これから印刷して送る場合は、宛名印刷フローをご利用ください。
          </p>
        ) : (
          <p className="address-form-help">宛名・差出人の差し替えはできません（削除して再登録）。</p>
        )}

        <div className="address-form-label">
          <span>宛名</span>
          <div>
            <span>{values.addressEntryDisplayName ?? '未選択'}</span>
            {allowIdentityEdit ? (
              <>
                <button
                  type="button"
                  id={POSTCARD_SEND_FIELD_IDS.addressEntryId}
                  className="link-button"
                  onClick={() => setAddressDialogOpen(true)}
                  aria-invalid={Boolean(errors.addressEntryId)}
                >
                  選択
                </button>
                {values.addressEntryId ? (
                  <button type="button" className="link-button" onClick={form.clearAddressEntry}>
                    クリア
                  </button>
                ) : null}
              </>
            ) : null}
          </div>
        </div>
        {errors.addressEntryId ? <p className="address-form-error">{errors.addressEntryId}</p> : null}

        <div className="address-form-label">
          <span>差出人</span>
          <div>
            <span>{values.senderEntryLabel ?? '未選択'}</span>
            {allowIdentityEdit ? (
              <>
                <button
                  type="button"
                  id={POSTCARD_SEND_FIELD_IDS.senderEntryId}
                  className="link-button"
                  onClick={() => setSenderDialogOpen(true)}
                  aria-invalid={Boolean(errors.senderEntryId)}
                >
                  選択
                </button>
                {values.senderEntryId ? (
                  <button type="button" className="link-button" onClick={form.clearSenderEntry}>
                    クリア
                  </button>
                ) : null}
              </>
            ) : null}
          </div>
        </div>
        {errors.senderEntryId ? <p className="address-form-error">{errors.senderEntryId}</p> : null}
      </fieldset>

      <div className="address-form-actions">
        <button type="button" className="link-button" onClick={onCancel} disabled={isSubmitting}>
          キャンセル
        </button>
        <button type="submit" className="address-list-create-button" disabled={isSubmitting}>
          {isSubmitting ? '保存中…' : submitLabel}
        </button>
      </div>

      <AddressEntrySelectDialog
        isOpen={addressDialogOpen}
        onClose={() => setAddressDialogOpen(false)}
        onSelect={(item: AddressEntryListItem) => {
          const label = formatDisplayName(item.primaryName, item.coRecipients)
          form.setAddressEntry(item.id, label)
          setAddressDialogOpen(false)
        }}
      />
      <SenderEntrySelectDialog
        isOpen={senderDialogOpen}
        selectedId={values.senderEntryId}
        onClose={() => setSenderDialogOpen(false)}
        onSelect={async (senderEntryId) => {
          try {
            const sender = await invoke<{ id: string; label: string }>('get_sender_entry', {
              id: senderEntryId,
            })
            form.setSenderEntry(sender.id, sender.label)
          } catch {
            form.setSenderEntry(senderEntryId, senderEntryId)
          }
          setSenderDialogOpen(false)
        }}
      />
    </form>
  )
}
