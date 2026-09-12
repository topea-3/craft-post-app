import { useCallback, useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { isKanaOnly } from '../address/kana'
import type { AddressFormValues, PersonNameForm } from '../address/types'
import {
  createEmptyPersonName,
  createInitialSenderEntryFormValues,
  SENDER_CO_RECIPIENTS_MAX,
  SENDER_LABEL_MAX_LENGTH,
  SENDER_PHONE_MAX_LENGTH,
  toSenderEntryDtoInput,
  type SenderEntryFormValues,
} from './types'
import { SENDER_OPERATION_ERROR_MESSAGE } from './messages'

export type SenderEntryFormErrors = {
  label?: string
  primaryName?: {
    last?: string
    first?: string
    kanaLast?: string
    kanaFirst?: string
  }
  coRecipients?: {
    [index: number]: {
      last?: string
      first?: string
      kanaLast?: string
      kanaFirst?: string
    }
  }
  address?: {
    postalCode?: string
    prefecture?: string
    city?: string
    street?: string
    building?: string
  }
  phoneNumber?: string
  form?: string
}

type UseSenderEntryFormResult = {
  values: SenderEntryFormValues
  errors: SenderEntryFormErrors
  isSubmitting: boolean
  isDirty: boolean
  updateLabel: (value: string) => void
  updatePrimaryName: (patch: Partial<PersonNameForm>) => void
  updateAddress: (patch: Partial<AddressFormValues>) => void
  addCoRecipient: () => void
  updateCoRecipient: (index: number, patch: Partial<PersonNameForm>) => void
  removeCoRecipient: (index: number) => void
  updatePhoneNumber: (value: string) => void
  submit: () => Promise<boolean>
}

const PERSON_NAME_MAX_LENGTH = 128
const ADDRESS_FIELD_MAX_LENGTH = 256

const validateSenderEntryForm = (current: SenderEntryFormValues): SenderEntryFormErrors => {
  const next: SenderEntryFormErrors = {}

  if (!current.label.trim()) {
    next.label = 'ラベルは必須です。'
  } else if (current.label.length > SENDER_LABEL_MAX_LENGTH) {
    next.label = `ラベルは${SENDER_LABEL_MAX_LENGTH}文字以内で入力してください。`
  }

  const primaryErrors: NonNullable<SenderEntryFormErrors['primaryName']> = {}
  if (!current.primaryName.last.trim()) {
    primaryErrors.last = '姓は必須です。'
  }
  if (!current.primaryName.first.trim()) {
    primaryErrors.first = '名は必須です。'
  }
  if (current.primaryName.last.length > PERSON_NAME_MAX_LENGTH) {
    primaryErrors.last = `姓は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
  }
  if (current.primaryName.first.length > PERSON_NAME_MAX_LENGTH) {
    primaryErrors.first = `名は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
  }
  if (current.primaryName.kanaLast && current.primaryName.kanaLast.length > PERSON_NAME_MAX_LENGTH) {
    primaryErrors.kanaLast = `カナ（姓）は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
  }
  if (current.primaryName.kanaFirst && current.primaryName.kanaFirst.length > PERSON_NAME_MAX_LENGTH) {
    primaryErrors.kanaFirst = `カナ（名）は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
  }
  if (current.primaryName.kanaLast && !isKanaOnly(current.primaryName.kanaLast)) {
    primaryErrors.kanaLast = 'カナ（姓）はひらがな・カタカナで入力してください。'
  }
  if (current.primaryName.kanaFirst && !isKanaOnly(current.primaryName.kanaFirst)) {
    primaryErrors.kanaFirst = 'カナ（名）はひらがな・カタカナで入力してください。'
  }
  if (Object.keys(primaryErrors).length > 0) {
    next.primaryName = primaryErrors
  }

  if (current.coRecipients.length > 0) {
    const coErrors: NonNullable<SenderEntryFormErrors['coRecipients']> = {}
    current.coRecipients.forEach((co, index) => {
      const e: NonNullable<SenderEntryFormErrors['primaryName']> = {}
      const lastTrim = co.last.trim()
      const firstTrim = co.first.trim()
      const hasAnyField =
        lastTrim.length > 0 ||
        firstTrim.length > 0 ||
        (co.kanaLast && co.kanaLast.trim().length > 0) ||
        (co.kanaFirst && co.kanaFirst.trim().length > 0)
      if (!hasAnyField) return
      if (!lastTrim) e.last = '連名の姓を入力するか、行を削除してください。'
      if (!firstTrim) e.first = '連名の名を入力するか、行を削除してください。'
      if (co.last.length > PERSON_NAME_MAX_LENGTH) {
        e.last = `姓は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
      }
      if (co.first.length > PERSON_NAME_MAX_LENGTH) {
        e.first = `名は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
      }
      if (co.kanaLast && co.kanaLast.length > PERSON_NAME_MAX_LENGTH) {
        e.kanaLast = `カナ（姓）は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
      }
      if (co.kanaFirst && co.kanaFirst.length > PERSON_NAME_MAX_LENGTH) {
        e.kanaFirst = `カナ（名）は${PERSON_NAME_MAX_LENGTH}文字以内で入力してください。`
      }
      if (co.kanaLast && !isKanaOnly(co.kanaLast)) {
        e.kanaLast = 'カナ（姓）はひらがな・カタカナで入力してください。'
      }
      if (co.kanaFirst && !isKanaOnly(co.kanaFirst)) {
        e.kanaFirst = 'カナ（名）はひらがな・カタカナで入力してください。'
      }
      if (Object.keys(e).length > 0) {
        coErrors[index] = e
      }
    })
    if (Object.keys(coErrors).length > 0) {
      next.coRecipients = coErrors
    }
  }

  const addrErrors: NonNullable<SenderEntryFormErrors['address']> = {}
  const { postalCode, prefecture, city, street, building } = current.address
  if (!postalCode.trim()) {
    addrErrors.postalCode = '郵便番号は必須です。'
  } else if (!/^\d{7}$/.test(postalCode)) {
    addrErrors.postalCode = '郵便番号は 7 桁の半角数字で入力してください。'
  }
  if (!prefecture.trim()) {
    addrErrors.prefecture = '都道府県は必須です。'
  }
  if (!city.trim()) {
    addrErrors.city = '市区町村は必須です。'
  } else if (city.length > ADDRESS_FIELD_MAX_LENGTH) {
    addrErrors.city = `市区町村は${ADDRESS_FIELD_MAX_LENGTH}文字以内で入力してください。`
  }
  if (!street.trim()) {
    addrErrors.street = '町名・番地は必須です。'
  } else if (street.length > ADDRESS_FIELD_MAX_LENGTH) {
    addrErrors.street = `町名・番地は${ADDRESS_FIELD_MAX_LENGTH}文字以内で入力してください。`
  }
  if (building && building.length > ADDRESS_FIELD_MAX_LENGTH) {
    addrErrors.building = `建物名・部屋番号は${ADDRESS_FIELD_MAX_LENGTH}文字以内で入力してください。`
  }
  if (Object.keys(addrErrors).length > 0) {
    next.address = addrErrors
  }

  if (current.phoneNumber && current.phoneNumber.length > SENDER_PHONE_MAX_LENGTH) {
    next.phoneNumber = `電話番号は${SENDER_PHONE_MAX_LENGTH}文字以内で入力してください。`
  }

  return next
}

export function useSenderEntryForm(onSuccess: () => void): UseSenderEntryFormResult {
  const initialValues = useMemo(() => createInitialSenderEntryFormValues(), [])
  return useSenderEntryFormBase(initialValues, async (dto) => {
    await invoke('create_sender_entry', { dto })
  }, onSuccess)
}

function useSenderEntryFormBase(
  initialValues: SenderEntryFormValues,
  submitToServer: (dto: ReturnType<typeof toSenderEntryDtoInput>) => Promise<void>,
  onSuccess: () => void,
): UseSenderEntryFormResult {
  const [values, setValues] = useState<SenderEntryFormValues>(initialValues)
  const [errors, setErrors] = useState<SenderEntryFormErrors>({})
  const [isSubmitting, setSubmitting] = useState(false)
  const [isDirty, setDirty] = useState(false)

  useEffect(() => {
    setValues(initialValues)
    setErrors({})
    setDirty(false)
  }, [initialValues])

  const setValue = (patch: Partial<SenderEntryFormValues>) => {
    setValues((prev) => ({ ...prev, ...patch }))
    setDirty(true)
  }

  const submit = async () => {
    const validation = validateSenderEntryForm(values)
    if (values.coRecipients.length > SENDER_CO_RECIPIENTS_MAX) {
      validation.form = `連名は${SENDER_CO_RECIPIENTS_MAX}件までです。`
    }
    setErrors(validation)
    if (Object.keys(validation).length > 0) {
      setErrors((prev) => ({
        ...prev,
        form: prev.form ?? '入力内容に不備があります。赤字の項目を修正してください。',
      }))
      return false
    }

    setSubmitting(true)
    try {
      const dto = toSenderEntryDtoInput(values)
      await submitToServer(dto)
      onSuccess()
      return true
    } catch (error) {
      console.error('Failed to submit sender entry:', error)
      const serverMessage =
        typeof error === 'string'
          ? error
          : error instanceof Error
            ? error.message
            : null
      setErrors((prev) => ({
        ...prev,
        form: serverMessage && serverMessage.trim().length > 0
          ? serverMessage
          : SENDER_OPERATION_ERROR_MESSAGE,
      }))
      return false
    } finally {
      setSubmitting(false)
    }
  }

  const updateLabel = useCallback((value: string) => {
    setValue({ label: value })
  }, [])
  const updatePrimaryName = (patch: Partial<PersonNameForm>) => {
    setValue({ primaryName: { ...values.primaryName, ...patch } })
  }
  const updateAddress = (patch: Partial<AddressFormValues>) => {
    setValue({ address: { ...values.address, ...patch } })
  }
  const addCoRecipient = () => {
    if (values.coRecipients.length >= SENDER_CO_RECIPIENTS_MAX) return
    setValue({ coRecipients: [...values.coRecipients, createEmptyPersonName()] })
  }
  const updateCoRecipient = (index: number, patch: Partial<PersonNameForm>) => {
    setValue({
      coRecipients: values.coRecipients.map((c, i) => (i === index ? { ...c, ...patch } : c)),
    })
  }
  const removeCoRecipient = (index: number) => {
    setValue({ coRecipients: values.coRecipients.filter((_, i) => i !== index) })
  }
  const updatePhoneNumber = (value: string) => setValue({ phoneNumber: value })

  return {
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
  }
}

export function useSenderEntryEditForm(
  id: string,
  initialValues: SenderEntryFormValues,
  onSuccess: () => void,
): UseSenderEntryFormResult {
  const submitToServer = useCallback(
    async (dto: ReturnType<typeof toSenderEntryDtoInput>) => {
      await invoke('update_sender_entry', { id, dto })
    },
    [id],
  )
  return useSenderEntryFormBase(initialValues, submitToServer, onSuccess)
}

