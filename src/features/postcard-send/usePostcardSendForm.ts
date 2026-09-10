import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { isFutureLocalDate } from '../../lib/date'
import type { PostcardSendFormValues, PostcardType } from './types'
import { createInitialPostcardSendFormValues } from './types'
import { mapPostcardSendInvokeError } from './messages'

export type PostcardSendFormErrors = {
  sentOn?: string
  postcardType?: string
  addressEntryId?: string
  senderEntryId?: string
  memo?: string
  form?: string
}

const MAX_MEMO_LENGTH = 1000

export const POSTCARD_SEND_FIELD_IDS = {
  sentOn: 'postcard-send-sent-on',
  postcardType: 'postcard-send-postcard-type',
  memo: 'postcard-send-memo',
  addressEntryId: 'postcard-send-address-entry',
  senderEntryId: 'postcard-send-sender-entry',
  form: 'postcard-send-form-error',
} as const

const ERROR_FOCUS_ORDER: (keyof PostcardSendFormErrors)[] = [
  'sentOn',
  'postcardType',
  'addressEntryId',
  'senderEntryId',
  'memo',
  'form',
]

function focusFirstError(errors: PostcardSendFormErrors) {
  for (const key of ERROR_FOCUS_ORDER) {
    if (!errors[key]) continue
    const id = POSTCARD_SEND_FIELD_IDS[key]
    const el = document.getElementById(id)
    if (el) {
      el.focus()
      return
    }
  }
}

export type UsePostcardSendFormResult = {
  values: PostcardSendFormValues
  errors: PostcardSendFormErrors
  isSubmitting: boolean
  isDirty: boolean
  /** 作成時のみ true。編集時は宛名・差出人固定 */
  allowIdentityEdit: boolean
  setAddressEntry: (id: string, displayName: string) => void
  clearAddressEntry: () => void
  setSenderEntry: (id: string, label: string) => void
  clearSenderEntry: () => void
  updateSentOn: (value: string) => void
  updatePostcardType: (value: PostcardType) => void
  updateMemo: (value: string) => void
  submit: () => Promise<boolean>
}

const validateForm = (
  values: PostcardSendFormValues,
  allowIdentityEdit: boolean,
  baselineSentOn?: string,
): PostcardSendFormErrors => {
  const errors: PostcardSendFormErrors = {}

  if (!values.sentOn.trim()) {
    errors.sentOn = '送付日を入力してください。'
  } else if (isFutureLocalDate(values.sentOn) && values.sentOn !== baselineSentOn) {
    errors.sentOn = '送付日に未来の日付は指定できません。'
  }

  if (!values.postcardType) {
    errors.postcardType = '種別を選択してください。'
  }

  if (allowIdentityEdit) {
    if (!values.addressEntryId) {
      errors.addressEntryId = '宛名を選択してください。'
    }
    if (!values.senderEntryId) {
      errors.senderEntryId = '差出人を選択してください。'
    }
  }

  if (Array.from(values.memo).length > MAX_MEMO_LENGTH) {
    errors.memo = `メモは${MAX_MEMO_LENGTH}文字以内で入力してください。`
  }

  return errors
}

type CreateArgs = {
  mode: 'create'
  onSuccess: (id: string) => void
}

type EditArgs = {
  mode: 'edit'
  sendId: string
  expectedUpdatedAt: string
  initialValues: PostcardSendFormValues
  baselineSentOn: string
  onSuccess: () => void
}

/**
 * 送付履歴フォーム。
 * create の初期値は useMemo で固定する（毎レンダー新規オブジェクト + sync effect は無限更新になる）。
 * edit の initialValues は親がロード完了後に一度だけ渡す想定（受取編集と同契約）。
 */
export function usePostcardSendForm(args: CreateArgs | EditArgs): UsePostcardSendFormResult {
  const allowIdentityEdit = args.mode === 'create'
  const createInitialValues = useMemo(() => createInitialPostcardSendFormValues(), [])
  const seedValues = args.mode === 'edit' ? args.initialValues : createInitialValues
  const editSyncKey = args.mode === 'edit' ? `${args.sendId}:${args.expectedUpdatedAt}` : null

  const [values, setValues] = useState<PostcardSendFormValues>(seedValues)
  const [errors, setErrors] = useState<PostcardSendFormErrors>({})
  const [isSubmitting, setSubmitting] = useState(false)
  const [isDirty, setDirty] = useState(false)
  const cancelledRef = useRef(false)
  const isSubmittingRef = useRef(false)
  const senderManuallySetRef = useRef(false)
  const lastEditSyncKeyRef = useRef<string | null>(editSyncKey)
  const argsRef = useRef(args)
  argsRef.current = args
  const valuesRef = useRef(values)
  valuesRef.current = values

  useEffect(() => {
    cancelledRef.current = false
    return () => {
      cancelledRef.current = true
    }
  }, [])

  // edit: 同一レコードの再同期のみ。オブジェクト参照の変化では回さない
  useEffect(() => {
    if (editSyncKey == null) return
    if (lastEditSyncKeyRef.current === editSyncKey) return
    lastEditSyncKeyRef.current = editSyncKey
    const current = argsRef.current
    if (current.mode !== 'edit') return
    setValues(current.initialValues)
    setErrors({})
    setDirty(false)
    senderManuallySetRef.current = false
  }, [editSyncKey])

  const patchValues = useCallback((patch: Partial<PostcardSendFormValues>) => {
    if (isSubmittingRef.current) return
    setValues((prev) => ({ ...prev, ...patch }))
    setDirty(true)
  }, [])

  const resolveLinkedSender = useCallback(async (addressEntryId: string) => {
    try {
      const senderId = await invoke<string | null>('get_sender_id_by_address_entry_id', {
        addressEntryId,
      })
      if (!senderId) {
        return null
      }
      const sender = await invoke<{ id: string; label: string }>('get_sender_entry', {
        id: senderId,
      })
      return { id: sender.id, label: sender.label }
    } catch {
      return null
    }
  }, [])

  const setAddressEntry = useCallback(
    (id: string, displayName: string) => {
      if (isSubmittingRef.current || !allowIdentityEdit) return
      const apply = async () => {
        const linked = await resolveLinkedSender(id)
        if (cancelledRef.current) return
        if (senderManuallySetRef.current && valuesRef.current.senderEntryId) {
          const confirmed = window.confirm(
            '宛名を変更すると差出人がリンク結果で上書きされます。よろしいですか？',
          )
          if (!confirmed) {
            patchValues({ addressEntryId: id, addressEntryDisplayName: displayName })
            return
          }
        }
        senderManuallySetRef.current = false
        patchValues({
          addressEntryId: id,
          addressEntryDisplayName: displayName,
          senderEntryId: linked?.id ?? null,
          senderEntryLabel: linked?.label ?? null,
        })
      }
      void apply()
    },
    [allowIdentityEdit, patchValues, resolveLinkedSender],
  )

  const clearAddressEntry = useCallback(() => {
    if (!allowIdentityEdit) return
    senderManuallySetRef.current = false
    patchValues({
      addressEntryId: null,
      addressEntryDisplayName: null,
      senderEntryId: null,
      senderEntryLabel: null,
    })
  }, [allowIdentityEdit, patchValues])

  const setSenderEntry = useCallback(
    (id: string, label: string) => {
      if (!allowIdentityEdit) return
      senderManuallySetRef.current = true
      patchValues({ senderEntryId: id, senderEntryLabel: label })
    },
    [allowIdentityEdit, patchValues],
  )

  const clearSenderEntry = useCallback(() => {
    if (!allowIdentityEdit) return
    senderManuallySetRef.current = true
    patchValues({ senderEntryId: null, senderEntryLabel: null })
  }, [allowIdentityEdit, patchValues])

  const updateSentOn = useCallback(
    (value: string) => patchValues({ sentOn: value }),
    [patchValues],
  )
  const updatePostcardType = useCallback(
    (value: PostcardType) => patchValues({ postcardType: value }),
    [patchValues],
  )
  const updateMemo = useCallback((value: string) => patchValues({ memo: value }), [patchValues])

  const submit = useCallback(async () => {
    if (isSubmittingRef.current) {
      return false
    }

    const currentArgs = argsRef.current
    const currentValues = valuesRef.current
    const currentBaseline =
      currentArgs.mode === 'edit' ? currentArgs.baselineSentOn : undefined
    const validation = validateForm(
      currentValues,
      currentArgs.mode === 'create',
      currentBaseline,
    )
    setErrors(validation)
    if (Object.keys(validation).length > 0) {
      queueMicrotask(() => focusFirstError(validation))
      return false
    }

    isSubmittingRef.current = true
    setSubmitting(true)
    try {
      if (currentArgs.mode === 'create') {
        const result = await invoke<{ ids: string[] }>('create_postcard_sends_manual_batch', {
          input: {
            postcard_type: currentValues.postcardType,
            sent_on: currentValues.sentOn,
            memo: currentValues.memo.trim() || null,
            items: [
              {
                address_entry_id: currentValues.addressEntryId,
                sender_entry_id: currentValues.senderEntryId,
              },
            ],
          },
        })
        if (cancelledRef.current) return false
        const id = result.ids[0]
        if (!id) {
          setErrors({ form: mapPostcardSendInvokeError('missing id') })
          return false
        }
        currentArgs.onSuccess(id)
        return true
      }

      await invoke('update_postcard_send', {
        id: currentArgs.sendId,
        input: {
          sent_on: currentValues.sentOn,
          postcard_type: currentValues.postcardType,
          memo: currentValues.memo.trim() || null,
          expected_updated_at: currentArgs.expectedUpdatedAt,
        },
      })
      if (cancelledRef.current) return false
      currentArgs.onSuccess()
      return true
    } catch (error) {
      if (cancelledRef.current) return false
      setErrors({ form: mapPostcardSendInvokeError(error) })
      queueMicrotask(() => focusFirstError({ form: 'x' }))
      return false
    } finally {
      isSubmittingRef.current = false
      if (!cancelledRef.current) {
        setSubmitting(false)
      }
    }
  }, [])

  return {
    values,
    errors,
    isSubmitting,
    isDirty,
    allowIdentityEdit,
    setAddressEntry,
    clearAddressEntry,
    setSenderEntry,
    clearSenderEntry,
    updateSentOn,
    updatePostcardType,
    updateMemo,
    submit,
  }
}
