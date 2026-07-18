import { useCallback, useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { isFutureLocalDate } from '../../lib/date'
import {
  type PostcardReceiptDtoInput,
  type PostcardReceiptFormValues,
  createInitialPostcardReceiptFormValues,
  toPostcardReceiptDtoInput,
} from './types'
import { POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE } from './messages'

export type PostcardReceiptFormErrors = {
  receivedAt?: string
  category?: string
  senderDisplayName?: string
  addressEntryId?: string
  memo?: string
  form?: string
}

const MAX_MEMO_LENGTH = 1000

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

const validateForm = (values: PostcardReceiptFormValues): PostcardReceiptFormErrors => {
  const errors: PostcardReceiptFormErrors = {}

  if (!values.receivedAt.trim()) {
    errors.receivedAt = '受取日を入力してください。'
  } else if (isFutureLocalDate(values.receivedAt)) {
    errors.receivedAt = '受取日に未来の日付は指定できません。'
  }

  if (values.linkMode === 'displayName' && !values.senderDisplayName.trim()) {
    errors.senderDisplayName = '送り主の表示名を入力してください。'
  }

  if (values.linkMode === 'address' && !values.addressEntryId) {
    errors.addressEntryId = '住所録から送り主を選択してください。'
  }

  if (values.memo.length > MAX_MEMO_LENGTH) {
    errors.memo = `メモは${MAX_MEMO_LENGTH}文字以内で入力してください。`
  }

  return errors
}

const usePostcardReceiptFormBase = (
  initialValues: PostcardReceiptFormValues,
  submitToServer: (dto: PostcardReceiptDtoInput) => Promise<void>,
  onSuccess: () => void,
): UsePostcardReceiptFormResult => {
  const [values, setValues] = useState<PostcardReceiptFormValues>(initialValues)
  const [errors, setErrors] = useState<PostcardReceiptFormErrors>({})
  const [isSubmitting, setSubmitting] = useState(false)
  const [isDirty, setDirty] = useState(false)

  useEffect(() => {
    setValues(initialValues)
    setErrors({})
    setDirty(false)
  }, [initialValues])

  const patchValues = (patch: Partial<PostcardReceiptFormValues>) => {
    setValues((prev) => ({ ...prev, ...patch }))
    setDirty(true)
  }

  const submit = async () => {
    const validation = validateForm(values)
    setErrors(validation)
    if (Object.keys(validation).length > 0) {
      return false
    }

    setSubmitting(true)
    try {
      const dto = toPostcardReceiptDtoInput(values)
      await submitToServer(dto)
      onSuccess()
      return true
    } catch (error) {
      console.error('Failed to submit postcard receipt:', error)
      const message =
        typeof error === 'string' && error.includes('address entry')
          ? '選択した宛名はアーカイブ済みです。'
          : typeof error === 'string'
            ? error
            : POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE
      setErrors((prev) => ({ ...prev, form: message }))
      return false
    } finally {
      setSubmitting(false)
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
  const submitToServer = useCallback(async (dto: PostcardReceiptDtoInput) => {
    const id = await invoke<string>('create_postcard_receipt', { dto })
    onSuccess(id)
  }, [onSuccess])
  return usePostcardReceiptFormBase(initialValues, submitToServer, () => {})
}

export function usePostcardReceiptEditForm(
  id: string,
  initialValues: PostcardReceiptFormValues,
  onSuccess: () => void,
): UsePostcardReceiptFormResult {
  const submitToServer = useCallback(
    async (dto: PostcardReceiptDtoInput) => {
      await invoke('update_postcard_receipt', { id, dto })
    },
    [id],
  )
  return usePostcardReceiptFormBase(initialValues, submitToServer, onSuccess)
}
