import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { isFutureLocalDate } from '../../lib/date'
import {
  type PostcardReceiptDtoInput,
  type PostcardReceiptFormValues,
  createInitialPostcardReceiptFormValues,
  toPostcardReceiptDtoInput,
} from './types'
import { mapPostcardReceiptInvokeError } from './messages'

export type PostcardReceiptFormErrors = {
  receivedAt?: string
  category?: string
  senderDisplayName?: string
  addressEntryId?: string
  memo?: string
  form?: string
}

const MAX_MEMO_LENGTH = 1000

export const POSTCARD_RECEIPT_FIELD_IDS = {
  receivedAt: 'postcard-receipt-received-at',
  category: 'postcard-receipt-category',
  memo: 'postcard-receipt-memo',
  addressEntryId: 'postcard-receipt-address-entry',
  senderDisplayName: 'postcard-receipt-sender-display-name',
  form: 'postcard-receipt-form-error',
} as const

const ERROR_FOCUS_ORDER: (keyof PostcardReceiptFormErrors)[] = [
  'receivedAt',
  'category',
  'senderDisplayName',
  'addressEntryId',
  'memo',
  'form',
]

function focusFirstPostcardReceiptError(errors: PostcardReceiptFormErrors) {
  for (const key of ERROR_FOCUS_ORDER) {
    if (!errors[key]) continue
    const id = POSTCARD_RECEIPT_FIELD_IDS[key]
    const el = document.getElementById(id)
    if (el) {
      el.focus()
      return
    }
  }
}

export type UsePostcardReceiptFormResult = {
  values: PostcardReceiptFormValues
  errors: PostcardReceiptFormErrors
  isSubmitting: boolean
  isDirty: boolean
  setLinkMode: (mode: PostcardReceiptFormValues['linkMode']) => void
  setAddressEntry: (id: string, displayName: string) => void
  clearAddressEntry: () => void
  updateReceivedAt: (value: string) => void
  updateCategory: (value: PostcardReceiptFormValues['category']) => void
  updateSenderDisplayName: (value: string) => void
  updateMemo: (value: string) => void
  submit: () => Promise<boolean>
}

const validateForm = (
  values: PostcardReceiptFormValues,
  baselineReceivedAt?: string,
): PostcardReceiptFormErrors => {
  const errors: PostcardReceiptFormErrors = {}

  if (!values.receivedAt.trim()) {
    errors.receivedAt = '受取日を入力してください。'
  } else if (
    isFutureLocalDate(values.receivedAt) &&
    values.receivedAt !== baselineReceivedAt
  ) {
    errors.receivedAt = '受取日に未来の日付は指定できません。'
  }

  if (values.linkMode === 'displayName' && !values.senderDisplayName.trim()) {
    errors.senderDisplayName = '送り主の表示名を入力してください。'
  }

  if (values.linkMode === 'address' && !values.addressEntryId) {
    errors.addressEntryId = '住所録から送り主を選択してください。'
  }

  // Rust 側の chars().count() と揃える（絵文字などは1文字として数える）
  if (Array.from(values.memo).length > MAX_MEMO_LENGTH) {
    errors.memo = `メモは${MAX_MEMO_LENGTH}文字以内で入力してください。`
  }

  return errors
}

const usePostcardReceiptFormBase = (
  initialValues: PostcardReceiptFormValues,
  submitToServer: (dto: PostcardReceiptDtoInput) => Promise<void>,
  onSuccess: () => void,
  baselineReceivedAt?: string,
): UsePostcardReceiptFormResult => {
  const [values, setValues] = useState<PostcardReceiptFormValues>(initialValues)
  const [errors, setErrors] = useState<PostcardReceiptFormErrors>({})
  const [isSubmitting, setSubmitting] = useState(false)
  const [isDirty, setDirty] = useState(false)
  const cancelledRef = useRef(false)
  const isSubmittingRef = useRef(false)

  useEffect(() => {
    cancelledRef.current = false
    return () => {
      cancelledRef.current = true
    }
  }, [])

  useEffect(() => {
    setValues(initialValues)
    setErrors({})
    setDirty(false)
  }, [initialValues])

  const patchValues = (patch: Partial<PostcardReceiptFormValues>) => {
    if (isSubmittingRef.current) return
    setValues((prev) => ({ ...prev, ...patch }))
    setDirty(true)
  }

  const submit = async () => {
    if (isSubmittingRef.current) {
      return false
    }

    const validation = validateForm(values, baselineReceivedAt)
    setErrors(validation)
    if (Object.keys(validation).length > 0) {
      queueMicrotask(() => focusFirstPostcardReceiptError(validation))
      return false
    }

    isSubmittingRef.current = true
    setSubmitting(true)
    try {
      const dto = toPostcardReceiptDtoInput(values)
      await submitToServer(dto)
      if (cancelledRef.current) {
        return false
      }
      onSuccess()
      return true
    } catch (error) {
      if (cancelledRef.current) {
        return false
      }
      console.error('Failed to submit postcard receipt:', error)
      const message = mapPostcardReceiptInvokeError(error)
      const nextErrors = { form: message }
      setErrors((prev) => ({ ...prev, ...nextErrors }))
      queueMicrotask(() => focusFirstPostcardReceiptError(nextErrors))
      return false
    } finally {
      isSubmittingRef.current = false
      if (!cancelledRef.current) {
        setSubmitting(false)
      }
    }
  }

  return {
    values,
    errors,
    isSubmitting,
    isDirty,
    setLinkMode: (linkMode) => patchValues({ linkMode }),
    setAddressEntry: (addressEntryId, addressEntryDisplayName) =>
      patchValues({ addressEntryId, addressEntryDisplayName }),
    clearAddressEntry: () => patchValues({ addressEntryId: null, addressEntryDisplayName: null }),
    updateReceivedAt: (receivedAt) => patchValues({ receivedAt }),
    updateCategory: (category) => patchValues({ category }),
    updateSenderDisplayName: (senderDisplayName) => patchValues({ senderDisplayName }),
    updateMemo: (memo) => patchValues({ memo }),
    submit,
  }
}

export function usePostcardReceiptForm(onSuccess: (id: string) => void): UsePostcardReceiptFormResult {
  const initialValues = useMemo(() => createInitialPostcardReceiptFormValues(), [])
  const createdIdRef = useRef<string | null>(null)
  const submitToServer = useCallback(async (dto: PostcardReceiptDtoInput) => {
    createdIdRef.current = await invoke<string>('create_postcard_receipt', { dto })
  }, [])
  return usePostcardReceiptFormBase(initialValues, submitToServer, () => {
    const id = createdIdRef.current
    if (id) onSuccess(id)
  })
}

export function usePostcardReceiptEditForm(
  id: string,
  initialValues: PostcardReceiptFormValues,
  expectedUpdatedAt: string,
  onSuccess: () => void,
): UsePostcardReceiptFormResult {
  const submitToServer = useCallback(
    async (dto: PostcardReceiptDtoInput) => {
      await invoke('update_postcard_receipt', {
        id,
        dto,
        expectedUpdatedAt,
      })
    },
    [id, expectedUpdatedAt],
  )
  return usePostcardReceiptFormBase(
    initialValues,
    submitToServer,
    onSuccess,
    initialValues.receivedAt,
  )
}
