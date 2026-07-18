import {
  currentLocalYear,
  formatDateOnlyDisplay,
  formatLocalDate,
  formatUtcToLocalDateTime,
} from '../../lib/date'
import type { PersonNameForm } from '../address/types'
import { formatDisplayName } from '../address/types'

export type PostcardReceiptCategory = 'nenga' | 'mochu' | 'other'

export const POSTCARD_RECEIPT_CATEGORY_OPTIONS: {
  value: PostcardReceiptCategory
  label: string
}[] = [
  { value: 'nenga', label: '年賀状' },
  { value: 'mochu', label: '喪中はがき' },
  { value: 'other', label: 'その他' },
]

// DTO 形は src-tauri/src/lib.rs の PostcardReceiptDtoInput / PostcardReceiptDto に対応させる
export type PostcardReceiptDto = {
  id: string
  address_entry_id: string | null
  sender_display_name: string | null
  received_at: string
  category: PostcardReceiptCategory
  memo: string | null
  created_at: string
  updated_at: string
  address_entry_display_name: string | null
  address_entry_address_line: string | null
  address_entry_archived: boolean | null
}

export type PostcardReceiptDtoInput = {
  address_entry_id: string | null
  sender_display_name: string | null
  received_at: string
  category: PostcardReceiptCategory
  memo: string | null
}

export type PostcardReceiptListItem = {
  id: string
  addressEntryId: string | null
  senderDisplayName: string | null
  receivedAt: string
  category: PostcardReceiptCategory
  memo: string | null
  addressEntryDisplayName: string | null
  addressEntryArchived: boolean | null
}

export type PostcardReceiptDetail = PostcardReceiptListItem & {
  createdAt: string
  updatedAt: string
  addressEntryAddressLine: string | null
}

export type SenderLinkMode = 'address' | 'displayName'

export type PostcardReceiptFormValues = {
  receivedAt: string
  category: PostcardReceiptCategory
  memo: string
  linkMode: SenderLinkMode
  addressEntryId: string | null
  addressEntryDisplayName: string | null
  senderDisplayName: string
}

export function categoryLabel(category: PostcardReceiptCategory): string {
  return POSTCARD_RECEIPT_CATEGORY_OPTIONS.find((o) => o.value === category)?.label ?? category
}

export function formatReceivedAt(value: string): string {
  return formatDateOnlyDisplay(value)
}

export const formatDateTime = formatUtcToLocalDateTime

export function resolveSenderDisplayName(item: {
  addressEntryId: string | null
  senderDisplayName: string | null
  addressEntryDisplayName: string | null
  addressEntryArchived?: boolean | null
}): string {
  if (item.addressEntryId && item.addressEntryDisplayName) {
    if (item.addressEntryArchived) {
      return `（アーカイブ済みの宛名）${item.addressEntryDisplayName}`
    }
    return item.addressEntryDisplayName
  }
  return item.senderDisplayName?.trim() || '—'
}

export function fromPostcardReceiptDto(dto: PostcardReceiptDto): PostcardReceiptListItem {
  return {
    id: dto.id,
    addressEntryId: dto.address_entry_id,
    senderDisplayName: dto.sender_display_name,
    receivedAt: dto.received_at,
    category: dto.category,
    memo: dto.memo,
    addressEntryDisplayName: dto.address_entry_display_name,
    addressEntryArchived: dto.address_entry_archived,
  }
}

export function fromPostcardReceiptDtoToDetail(dto: PostcardReceiptDto): PostcardReceiptDetail {
  return {
    ...fromPostcardReceiptDto(dto),
    createdAt: dto.created_at,
    updatedAt: dto.updated_at,
    addressEntryAddressLine: dto.address_entry_address_line,
  }
}

export function toPostcardReceiptDtoInput(values: PostcardReceiptFormValues): PostcardReceiptDtoInput {
  const address_entry_id = values.linkMode === 'address' ? values.addressEntryId : null
  const sender_display_name =
    values.linkMode === 'displayName' ? values.senderDisplayName.trim() || null : null

  return {
    address_entry_id,
    sender_display_name,
    received_at: values.receivedAt,
    category: values.category,
    memo: values.memo.trim() || null,
  }
}

export function createInitialPostcardReceiptFormValues(): PostcardReceiptFormValues {
  const today = new Date()
  const month = today.getMonth() + 1
  const defaultCategory: PostcardReceiptCategory =
    month === 1 || month === 2 ? 'nenga' : 'other'

  return {
    receivedAt: formatLocalDate(today),
    category: defaultCategory,
    memo: '',
    linkMode: 'address',
    addressEntryId: null,
    addressEntryDisplayName: null,
    senderDisplayName: '',
  }
}

export function formValuesFromDetail(detail: PostcardReceiptDetail): PostcardReceiptFormValues {
  if (detail.addressEntryId) {
    return {
      receivedAt: detail.receivedAt,
      category: detail.category,
      memo: detail.memo ?? '',
      linkMode: 'address',
      addressEntryId: detail.addressEntryId,
      addressEntryDisplayName: detail.addressEntryDisplayName,
      senderDisplayName: detail.senderDisplayName ?? '',
    }
  }
  return {
    receivedAt: detail.receivedAt,
    category: detail.category,
    memo: detail.memo ?? '',
    linkMode: 'displayName',
    addressEntryId: null,
    addressEntryDisplayName: null,
    senderDisplayName: detail.senderDisplayName ?? '',
  }
}

/** 住所録の primary/co から表示名を組み立てる（選択ダイアログ用） */
export function formatAddressEntryLabel(
  primaryName: PersonNameForm,
  coRecipients: PersonNameForm[],
  honorific: string,
): string {
  const base = formatDisplayName(primaryName, coRecipients)
  return honorific ? `${base} ${honorific}`.trim() : base
}

export function buildYearOptions(): { value: string; label: string }[] {
  const currentYear = currentLocalYear()
  const years: { value: string; label: string }[] = [{ value: '', label: '全期間' }]
  for (let y = currentYear; y >= currentYear - 10; y -= 1) {
    years.push({ value: String(y), label: String(y) })
  }
  return years
}
