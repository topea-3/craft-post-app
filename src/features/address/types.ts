import { formatUtcToLocalDateTime } from '../../lib/date'

export type PersonNameForm = {
  last: string
  first: string
  kanaLast?: string
  kanaFirst?: string
}

export type AddressFormValues = {
  postalCode: string
  prefecture: string
  city: string
  street: string
  building?: string
}

export type AddressEntryFormValues = {
  primaryName: PersonNameForm
  coRecipients: PersonNameForm[]
  honorific: string
  address: AddressFormValues
  memo?: string
}

export type AddressEntryListItem = {
  id: string
  primaryName: PersonNameForm
  coRecipients: PersonNameForm[]
  honorific: string
  postalCode: string
  address: AddressFormValues
  memo?: string
  updatedAt: string
  archived: boolean
}

export type AddressEntryDetail = AddressEntryListItem & {
  createdAt: string
}

export type AddressEntryDto = {
  id: string
  primary_name: PersonNameDto
  co_recipients: PersonNameDto[]
  honorific: string
  postal_code: string
  address: AddressDto
  memo?: string | null
  archived: boolean
  created_at: string
  updated_at: string
}

// DTO 形は src-tauri/src/lib.rs の AddressEntryDtoInput, PersonNameDto, AddressDto に対応させる

export type PersonNameDto = {
  last: string
  first: string
  kana_last?: string | null
  kana_first?: string | null
}

export type AddressDto = {
  prefecture: string
  city: string
  street: string
  building?: string | null
}

export type AddressEntryDtoInput = {
  primary_name: PersonNameDto
  co_recipients: PersonNameDto[]
  honorific: string
  postal_code: string
  address: AddressDto
  memo?: string | null
}

export const HONORIFIC_OPTIONS: { value: string; label: string }[] = [
  { value: '様', label: '様' },
  { value: '御中', label: '御中' },
  { value: 'ご家族様', label: 'ご家族様' },
  { value: 'なし', label: 'なし' },
]

export const PREFECTURE_OPTIONS: { value: string; label: string }[] = [
  { value: '', label: '選択してください' },
  { value: '北海道', label: '北海道' },
  { value: '青森県', label: '青森県' },
  { value: '岩手県', label: '岩手県' },
  { value: '宮城県', label: '宮城県' },
  { value: '秋田県', label: '秋田県' },
  { value: '山形県', label: '山形県' },
  { value: '福島県', label: '福島県' },
  { value: '茨城県', label: '茨城県' },
  { value: '栃木県', label: '栃木県' },
  { value: '群馬県', label: '群馬県' },
  { value: '埼玉県', label: '埼玉県' },
  { value: '千葉県', label: '千葉県' },
  { value: '東京都', label: '東京都' },
  { value: '神奈川県', label: '神奈川県' },
  { value: '新潟県', label: '新潟県' },
  { value: '富山県', label: '富山県' },
  { value: '石川県', label: '石川県' },
  { value: '福井県', label: '福井県' },
  { value: '山梨県', label: '山梨県' },
  { value: '長野県', label: '長野県' },
  { value: '岐阜県', label: '岐阜県' },
  { value: '静岡県', label: '静岡県' },
  { value: '愛知県', label: '愛知県' },
  { value: '三重県', label: '三重県' },
  { value: '滋賀県', label: '滋賀県' },
  { value: '京都府', label: '京都府' },
  { value: '大阪府', label: '大阪府' },
  { value: '兵庫県', label: '兵庫県' },
  { value: '奈良県', label: '奈良県' },
  { value: '和歌山県', label: '和歌山県' },
  { value: '鳥取県', label: '鳥取県' },
  { value: '島根県', label: '島根県' },
  { value: '岡山県', label: '岡山県' },
  { value: '広島県', label: '広島県' },
  { value: '山口県', label: '山口県' },
  { value: '徳島県', label: '徳島県' },
  { value: '香川県', label: '香川県' },
  { value: '愛媛県', label: '愛媛県' },
  { value: '高知県', label: '高知県' },
  { value: '福岡県', label: '福岡県' },
  { value: '佐賀県', label: '佐賀県' },
  { value: '長崎県', label: '長崎県' },
  { value: '熊本県', label: '熊本県' },
  { value: '大分県', label: '大分県' },
  { value: '宮崎県', label: '宮崎県' },
  { value: '鹿児島県', label: '鹿児島県' },
  { value: '沖縄県', label: '沖縄県' },
]

export const createEmptyPersonName = (): PersonNameForm => ({
  last: '',
  first: '',
  kanaLast: '',
  kanaFirst: '',
})

export const createInitialAddressEntryFormValues = (): AddressEntryFormValues => ({
  primaryName: createEmptyPersonName(),
  coRecipients: [],
  honorific: '様',
  address: {
    postalCode: '',
    prefecture: '',
    city: '',
    street: '',
    building: '',
  },
  memo: '',
})

/** 送信前の正規化: 前後空白を除去し、空のオプション文字列は null にする */
function trimOptional(value: string | undefined): string | null {
  const s = (value ?? '').trim()
  return s === '' ? null : s
}

export const toPersonNameDto = (form: PersonNameForm): PersonNameDto => ({
  last: form.last.trim(),
  first: form.first.trim(),
  kana_last: trimOptional(form.kanaLast),
  kana_first: trimOptional(form.kanaFirst),
})

export const toAddressEntryDtoInput = (form: AddressEntryFormValues): AddressEntryDtoInput => ({
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
  honorific: form.honorific.trim(),
  postal_code: form.address.postalCode.trim(),
  address: {
    prefecture: form.address.prefecture.trim(),
    city: form.address.city.trim(),
    street: form.address.street.trim(),
    building: trimOptional(form.address.building),
  },
  memo: trimOptional(form.memo),
})

export const fromAddressEntryDto = (dto: AddressEntryDto): AddressEntryListItem => ({
  id: dto.id,
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
  honorific: dto.honorific,
  postalCode: dto.postal_code,
  address: {
    postalCode: dto.postal_code,
    prefecture: dto.address.prefecture,
    city: dto.address.city,
    street: dto.address.street,
    building: dto.address.building ?? undefined,
  },
  memo: dto.memo ?? undefined,
  updatedAt: dto.updated_at,
  archived: dto.archived,
})

export const fromAddressEntryDtoToDetail = (
  dto: AddressEntryDto,
): AddressEntryDetail => ({
  ...fromAddressEntryDto(dto),
  createdAt: dto.created_at,
})

export const formatDisplayName = (
  primaryName: PersonNameForm,
  coRecipients: PersonNameForm[],
): string => {
  const primaryLast = primaryName.last.trim()
  const primaryFirst = primaryName.first.trim()
  const primaryFull = `${primaryLast} ${primaryFirst}`.trim()
  if (coRecipients.length === 0) return primaryFull

  const sameLastName = coRecipients.every(
    (co) => co.last.trim() === primaryLast,
  )
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

export const formatPostalCode = (postalCode: string): string => {
  const trimmed = postalCode.replace(/[^0-9]/g, '')
  if (trimmed.length !== 7) return postalCode
  return `${trimmed.slice(0, 3)}-${trimmed.slice(3)}`
}

export const formatAddressSingleLine = (address: AddressFormValues): string => {
  const { prefecture, city, street, building } = address
  const base = `${prefecture}${city}${street}`.trim()
  if (!building) return base
  return `${base} ${building}`.trim()
}

export const formatDateTime = formatUtcToLocalDateTime

export const formatUpdatedAt = (updatedAt: string): string => {
  return formatDateTime(updatedAt)
}

