import type { AddressFormValues, PersonNameForm } from '../address/types'
import { PREFECTURE_OPTIONS } from '../address/types'

export type SenderEntryFormValues = {
  label: string
  primaryName: PersonNameForm
  coRecipients: PersonNameForm[]
  address: AddressFormValues
  phoneNumber?: string
}

export type SenderEntryDtoInput = {
  label: string
  primary_name: {
    last: string
    first: string
    kana_last?: string | null
    kana_first?: string | null
  }
  co_recipients: {
    last: string
    first: string
    kana_last?: string | null
    kana_first?: string | null
  }[]
  postal_code: string
  address: {
    prefecture: string
    city: string
    street: string
    building?: string | null
  }
  phone_number?: string | null
}

export type SenderEntryDto = {
  id: string
  label: string
  primary_name: {
    last: string
    first: string
    kana_last?: string | null
    kana_first?: string | null
  }
  co_recipients: {
    last: string
    first: string
    kana_last?: string | null
    kana_first?: string | null
  }[]
  postal_code: string
  address: {
    prefecture: string
    city: string
    street: string
    building?: string | null
  }
  phone_number?: string | null
  archived: boolean
  created_at: string
  updated_at: string
}

export type SenderEntryListItem = {
  id: string
  label: string
  primaryName: PersonNameForm
  coRecipients: PersonNameForm[]
  address: AddressFormValues
  postalCode: string
  updatedAt: string
  archived: boolean
}

export type SenderEntryDetail = SenderEntryListItem & {
  phoneNumber?: string
  createdAt: string
}

export const SENDER_LABEL_MAX_LENGTH = 250
export const SENDER_CO_RECIPIENTS_MAX = 4
export const SENDER_PHONE_MAX_LENGTH = 32

export const createEmptyPersonName = (): PersonNameForm => ({
  last: '',
  first: '',
  kanaLast: '',
  kanaFirst: '',
})

export const createInitialSenderEntryFormValues = (): SenderEntryFormValues => ({
  label: '',
  primaryName: createEmptyPersonName(),
  coRecipients: [],
  address: {
    postalCode: '',
    prefecture: '',
    city: '',
    street: '',
    building: '',
  },
  phoneNumber: '',
})

function trimOptional(value: string | undefined): string | null {
  const s = (value ?? '').trim()
  return s === '' ? null : s
}

function toPersonNameDto(form: PersonNameForm) {
  return {
    last: form.last.trim(),
    first: form.first.trim(),
    kana_last: trimOptional(form.kanaLast),
    kana_first: trimOptional(form.kanaFirst),
  }
}

export const toSenderEntryDtoInput = (form: SenderEntryFormValues): SenderEntryDtoInput => ({
  label: form.label.trim(),
  primary_name: toPersonNameDto(form.primaryName),
  co_recipients: form.coRecipients
    .filter((co) => {
      const last = co.last.trim()
      const first = co.first.trim()
      const kanaLast = co.kanaLast?.trim()
      const kanaFirst = co.kanaFirst?.trim()
      return Boolean(last || first || kanaLast || kanaFirst)
    })
    .map(toPersonNameDto),
  postal_code: form.address.postalCode.trim(),
  address: {
    prefecture: form.address.prefecture.trim(),
    city: form.address.city.trim(),
    street: form.address.street.trim(),
    building: trimOptional(form.address.building),
  },
  phone_number: trimOptional(form.phoneNumber),
})

export const formatSenderDisplayName = (
  primaryName: PersonNameForm,
  coRecipients: PersonNameForm[],
): string => {
  const primaryLast = primaryName.last.trim()
  const primaryFirst = primaryName.first.trim()
  const primaryFull = `${primaryLast} ${primaryFirst}`.trim()
  if (coRecipients.length === 0) return primaryFull

  const sameLastName = coRecipients.every((co) => co.last.trim() === primaryLast)
  const coNames = coRecipients
    .map((co) => {
      const last = co.last.trim()
      const first = co.first.trim()
      if (!last && !first) return ''
      if (sameLastName) return first
      return `${last} ${first}`.trim()
    })
    .filter((name) => name !== '')

  return [primaryFull, ...coNames].join('・')
}

export const formatSenderOptionLabel = (
  sender: Pick<SenderEntryListItem, 'label' | 'primaryName' | 'coRecipients'>,
): string =>
  `${sender.label}（${formatSenderDisplayName(sender.primaryName, sender.coRecipients) || '—'}）`

export const fromSenderEntryDto = (dto: SenderEntryDto): SenderEntryListItem => ({
  id: dto.id,
  label: dto.label,
  primaryName: {
    last: dto.primary_name.last,
    first: dto.primary_name.first,
    kanaLast: dto.primary_name.kana_last ?? undefined,
    kanaFirst: dto.primary_name.kana_first ?? undefined,
  },
  coRecipients: dto.co_recipients.map((co) => ({
    last: co.last,
    first: co.first,
    kanaLast: co.kana_last ?? undefined,
    kanaFirst: co.kana_first ?? undefined,
  })),
  address: {
    postalCode: dto.postal_code,
    prefecture: dto.address.prefecture,
    city: dto.address.city,
    street: dto.address.street,
    building: dto.address.building ?? undefined,
  },
  postalCode: dto.postal_code,
  updatedAt: dto.updated_at,
  archived: dto.archived,
})

export const fromSenderEntryDtoToDetail = (dto: SenderEntryDto): SenderEntryDetail => ({
  ...fromSenderEntryDto(dto),
  phoneNumber: dto.phone_number ?? undefined,
  createdAt: dto.created_at,
})

export const formatSenderKanaDisplayName = (
  primaryName: PersonNameForm,
  coRecipients: PersonNameForm[],
): string => {
  const primaryLast = (primaryName.kanaLast ?? '').trim()
  const primaryFirst = (primaryName.kanaFirst ?? '').trim()
  const primaryFull = `${primaryLast} ${primaryFirst}`.trim()
  if (!primaryFull) return ''
  if (coRecipients.length === 0) return primaryFull

  const sameLastName = coRecipients.every((co) => (co.kanaLast ?? '').trim() === primaryLast)
  const coNames = coRecipients
    .map((co) => {
      const last = (co.kanaLast ?? '').trim()
      const first = (co.kanaFirst ?? '').trim()
      if (!last && !first) return ''
      if (sameLastName) return first
      return `${last} ${first}`.trim()
    })
    .filter((name) => name !== '')

  return [primaryFull, ...coNames].join('・')
}

export { PREFECTURE_OPTIONS }

