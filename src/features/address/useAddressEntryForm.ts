import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  type AddressEntryFormValues,
  type AddressFormValues,
  type PersonNameForm,
  createEmptyPersonName,
  createInitialAddressEntryFormValues,
  toAddressEntryDtoInput,
} from './types'

export type AddressEntryFormErrors = {
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
  honorific?: string
  address?: {
    postalCode?: string
    prefecture?: string
    city?: string
    street?: string
    building?: string
  }
  memo?: string
  form?: string
}

type UseAddressEntryFormResult = {
  values: AddressEntryFormValues
  errors: AddressEntryFormErrors
  isSubmitting: boolean
  isDirty: boolean
  updatePrimaryName: (patch: Partial<PersonNameForm>) => void
  updateAddress: (patch: Partial<AddressFormValues>) => void
  addCoRecipient: () => void
  updateCoRecipient: (index: number, patch: Partial<PersonNameForm>) => void
  removeCoRecipient: (index: number) => void
  updateHonorific: (value: string) => void
  updateMemo: (value: string) => void
  submit: () => Promise<boolean>
}

const MAX_TEXT_LENGTH = 256
const MAX_MEMO_LENGTH = 1000

export function useAddressEntryForm(onSuccess: () => void): UseAddressEntryFormResult {
  const [values, setValues] = useState<AddressEntryFormValues>(createInitialAddressEntryFormValues)
  const [errors, setErrors] = useState<AddressEntryFormErrors>({})
  const [isSubmitting, setSubmitting] = useState(false)
  const [isDirty, setDirty] = useState(false)

  const setValue = (patch: Partial<AddressEntryFormValues>) => {
    setValues((prev) => ({ ...prev, ...patch }))
    setDirty(true)
  }

  const validate = (current: AddressEntryFormValues): AddressEntryFormErrors => {
    const next: AddressEntryFormErrors = {}

    // primary name
    const primaryErrors: NonNullable<AddressEntryFormErrors['primaryName']> = {}
    if (!current.primaryName.last.trim()) {
      primaryErrors.last = '姓は必須です。'
    }
    if (!current.primaryName.first.trim()) {
      primaryErrors.first = '名は必須です。'
    }
    if (current.primaryName.last.length > MAX_TEXT_LENGTH) {
      primaryErrors.last = `姓は${MAX_TEXT_LENGTH}文字以内で入力してください。`
    }
    if (current.primaryName.first.length > MAX_TEXT_LENGTH) {
      primaryErrors.first = `名は${MAX_TEXT_LENGTH}文字以内で入力してください。`
    }
    if (current.primaryName.kanaLast && current.primaryName.kanaLast.length > MAX_TEXT_LENGTH) {
      primaryErrors.kanaLast = `カナ（姓）は${MAX_TEXT_LENGTH}文字以内で入力してください。`
    }
    if (current.primaryName.kanaFirst && current.primaryName.kanaFirst.length > MAX_TEXT_LENGTH) {
      primaryErrors.kanaFirst = `カナ（名）は${MAX_TEXT_LENGTH}文字以内で入力してください。`
    }
    if (Object.keys(primaryErrors).length > 0) {
      next.primaryName = primaryErrors
    }

    // co recipients
    if (current.coRecipients.length > 0) {
      const coErrors: NonNullable<AddressEntryFormErrors['coRecipients']> = {}
      current.coRecipients.forEach((co, index) => {
        const e: NonNullable<AddressEntryFormErrors['primaryName']> = {}
        const lastTrim = co.last.trim()
        const firstTrim = co.first.trim()
        const hasAnyField =
          lastTrim.length > 0 ||
          firstTrim.length > 0 ||
          (co.kanaLast && co.kanaLast.trim().length > 0) ||
          (co.kanaFirst && co.kanaFirst.trim().length > 0)

        if (!hasAnyField) {
          // 完全に空の連名行は「未使用行」とみなしてバリデーションしない。
          return
        }

        if (!lastTrim) {
          e.last = '連名の姓を入力するか、行を削除してください。'
        }
        if (!firstTrim) {
          e.first = '連名の名を入力するか、行を削除してください。'
        }
        if (co.last.length > MAX_TEXT_LENGTH) {
          e.last = `姓は${MAX_TEXT_LENGTH}文字以内で入力してください。`
        }
        if (co.first.length > MAX_TEXT_LENGTH) {
          e.first = `名は${MAX_TEXT_LENGTH}文字以内で入力してください。`
        }
        if (co.kanaLast && co.kanaLast.length > MAX_TEXT_LENGTH) {
          e.kanaLast = `カナ（姓）は${MAX_TEXT_LENGTH}文字以内で入力してください。`
        }
        if (co.kanaFirst && co.kanaFirst.length > MAX_TEXT_LENGTH) {
          e.kanaFirst = `カナ（名）は${MAX_TEXT_LENGTH}文字以内で入力してください。`
        }
        if (Object.keys(e).length > 0) {
          coErrors[index] = e
        }
      })
      if (Object.keys(coErrors).length > 0) {
        next.coRecipients = coErrors
      }
    }

    // address
    const addrErrors: NonNullable<AddressEntryFormErrors['address']> = {}
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
    } else if (city.length > MAX_TEXT_LENGTH) {
      addrErrors.city = `市区町村は${MAX_TEXT_LENGTH}文字以内で入力してください。`
    }
    if (!street.trim()) {
      addrErrors.street = '町名・番地は必須です。'
    } else if (street.length > MAX_TEXT_LENGTH) {
      addrErrors.street = `町名・番地は${MAX_TEXT_LENGTH}文字以内で入力してください。`
    }
    if (building && building.length > MAX_TEXT_LENGTH) {
      addrErrors.building = `建物名・部屋番号は${MAX_TEXT_LENGTH}文字以内で入力してください。`
    }
    if (Object.keys(addrErrors).length > 0) {
      next.address = addrErrors
    }

    // memo
    if (current.memo && current.memo.length > MAX_MEMO_LENGTH) {
      next.memo = `メモは${MAX_MEMO_LENGTH}文字以内で入力してください。`
    }

    return next
  }

  const submit = async () => {
    const validation = validate(values)
    setErrors(validation)
    if (Object.keys(validation).length > 0) {
      return false
    }

    setSubmitting(true)
    try {
      const dto = toAddressEntryDtoInput(values)
      await invoke('create_address_entry', { dto })
      onSuccess()
      return true
    } catch (e) {
      // サーバー側のバリデーションなどはフォーム全体エラーとして表示
      setErrors((prev) => ({
        ...prev,
        form: String(e),
      }))
      return false
    } finally {
      setSubmitting(false)
    }
  }

  const updatePrimaryName = (patch: Partial<PersonNameForm>) => {
    setValue({
      primaryName: { ...values.primaryName, ...patch },
    })
  }

  const updateAddress = (patch: Partial<AddressFormValues>) => {
    setValue({
      address: { ...values.address, ...patch },
    })
  }

  const addCoRecipient = () => {
    if (values.coRecipients.length >= 3) return
    setValue({
      coRecipients: [...values.coRecipients, createEmptyPersonName()],
    })
  }

  const updateCoRecipient = (index: number, patch: Partial<PersonNameForm>) => {
    setValue({
      coRecipients: values.coRecipients.map((c, i) =>
        i === index
          ? {
              ...c,
              ...patch,
            }
          : c,
      ),
    })
  }

  const removeCoRecipient = (index: number) => {
    setValue({
      coRecipients: values.coRecipients.filter((_, i) => i !== index),
    })
  }

  const updateHonorific = (value: string) => {
    setValue({ honorific: value })
  }

  const updateMemo = (value: string) => {
    setValue({ memo: value })
  }

  return {
    values,
    errors,
    isSubmitting,
    isDirty,
    updatePrimaryName,
    updateAddress,
    addCoRecipient,
    updateCoRecipient,
    removeCoRecipient,
    updateHonorific,
    updateMemo,
    submit,
  }
}

