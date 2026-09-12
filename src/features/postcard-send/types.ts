import {
  currentLocalYear,
  formatDateOnlyDisplay,
  formatLocalDate,
  formatUtcToLocalDateTime,
} from '../../lib/date'
import {
  formatRecipientDisplayName,
  formatSenderDisplayNameFromSnapshot,
  fromAddressPrintSnapshotDto,
  fromSenderPrintSnapshotDto,
  type AddressPrintSnapshot,
  type AddressPrintSnapshotDto,
  type SenderPrintSnapshot,
  type SenderPrintSnapshotDto,
} from '../print/types'

/** 検索キーワード上限（Unicode scalar）。受取 search と揃える想定 */
export const MAX_SEARCH_KEYWORD_LENGTH = 100

export type PostcardType = 'nenga' | 'mochu'
export type PostcardSendSource = 'print' | 'manual'
export type SendStatusFilter = 'sent' | 'unsent'

export const POSTCARD_TYPE_OPTIONS: { value: PostcardType; label: string }[] = [
  { value: 'nenga', label: '年賀状' },
  { value: 'mochu', label: '喪中はがき' },
]

export const POSTCARD_SEND_SOURCE_OPTIONS: { value: PostcardSendSource; label: string }[] = [
  { value: 'print', label: '印刷' },
  { value: 'manual', label: '手入力' },
]

/** DTO 形は src-tauri/src/lib.rs の PostcardSendDto に対応 */
export type PostcardSendDto = {
  id: string
  print_job_id: string
  address_entry_id: string
  sender_entry_id: string
  sender_snapshot?: string
  address_snapshot?: string
  postcard_type: string
  sent_on: string
  source: string
  memo: string | null
  created_at: string
  updated_at: string
  address_entry_display_name: string | null
  address_entry_address_line: string | null
  address_entry_archived: boolean | null
  sender_entry_label: string | null
  sender_entry_display_name: string | null
  sender_entry_archived: boolean | null
}

export type PostcardSendListItem = {
  id: string
  printJobId: string
  addressEntryId: string
  senderEntryId: string
  postcardType: PostcardType
  sentOn: string
  source: PostcardSendSource
  memo: string | null
  addressEntryDisplayName: string | null
  addressEntryArchived: boolean | null
  senderEntryLabel: string | null
  senderEntryDisplayName: string | null
  senderEntryArchived: boolean | null
  addressSnapshot: AddressPrintSnapshot | null
  senderSnapshot: SenderPrintSnapshot | null
}

export type PostcardSendDetail = PostcardSendListItem & {
  createdAt: string
  updatedAt: string
  addressEntryAddressLine: string | null
}

function parseAddressSnapshot(raw: string | undefined | null): AddressPrintSnapshot | null {
  if (!raw?.trim()) return null
  try {
    const parsed = JSON.parse(raw) as AddressPrintSnapshotDto
    if (!parsed || typeof parsed !== 'object') return null
    if (typeof parsed.primary_last !== 'string' || typeof parsed.primary_first !== 'string') {
      return null
    }
    return fromAddressPrintSnapshotDto({
      address_entry_id: parsed.address_entry_id ?? '',
      postal_code: parsed.postal_code ?? '',
      address_line1: parsed.address_line1 ?? '',
      address_line2: parsed.address_line2 ?? '',
      address_line3: parsed.address_line3 ?? '',
      primary_last: parsed.primary_last,
      primary_first: parsed.primary_first,
      co_recipients: Array.isArray(parsed.co_recipients) ? parsed.co_recipients : [],
      honorific_print: parsed.honorific_print ?? '',
    })
  } catch {
    return null
  }
}

function parseSenderSnapshot(raw: string | undefined | null): SenderPrintSnapshot | null {
  if (!raw?.trim()) return null
  try {
    const parsed = JSON.parse(raw) as SenderPrintSnapshotDto
    if (!parsed || typeof parsed !== 'object') return null
    if (typeof parsed.primary_last !== 'string' || typeof parsed.primary_first !== 'string') {
      return null
    }
    return fromSenderPrintSnapshotDto({
      sender_entry_id: parsed.sender_entry_id ?? '',
      postal_code: parsed.postal_code ?? '',
      address_line1: parsed.address_line1 ?? '',
      address_line2: parsed.address_line2 ?? '',
      address_line3: parsed.address_line3 ?? '',
      primary_last: parsed.primary_last,
      primary_first: parsed.primary_first,
      co_recipients: Array.isArray(parsed.co_recipients) ? parsed.co_recipients : [],
    })
  } catch {
    return null
  }
}

function formatSnapshotAddressLine(snapshot: AddressPrintSnapshot | SenderPrintSnapshot): string {
  return [snapshot.addressLine1, snapshot.addressLine2, snapshot.addressLine3]
    .map((part) => part.trim())
    .filter(Boolean)
    .join('')
}

function formatPostalCode(code: string): string {
  const digits = code.replace(/\D/g, '')
  if (digits.length === 7) {
    return `〒${digits.slice(0, 3)}-${digits.slice(3)}`
  }
  return code.trim() ? `〒${code.trim()}` : ''
}

export type PostcardSendFormValues = {
  sentOn: string
  postcardType: PostcardType
  memo: string
  addressEntryId: string | null
  addressEntryDisplayName: string | null
  senderEntryId: string | null
  senderEntryLabel: string | null
}

export type SendStatusItemDto = {
  address_entry_id: string
  display_name: string
  address_summary: string
  last_sent_on: string | null
  send_count: number
}

export type SendStatusItem = {
  addressEntryId: string
  displayName: string
  addressSummary: string
  lastSentOn: string | null
  sendCount: number
}

/** SND006 へ渡す location state */
export type BulkSendLocationState = {
  addressEntryIds?: string[]
  postcardType?: PostcardType | null
  fromStatusTab?: boolean
}

export function postcardTypeLabel(type: PostcardType): string {
  return POSTCARD_TYPE_OPTIONS.find((o) => o.value === type)?.label ?? type
}

export function sourceLabel(source: PostcardSendSource): string {
  return POSTCARD_SEND_SOURCE_OPTIONS.find((o) => o.value === source)?.label ?? source
}

export function formatSentOn(value: string): string {
  return formatDateOnlyDisplay(value)
}

export const formatDateTime = formatUtcToLocalDateTime

/** メモ抜粋（Unicode scalar 単位） */
export function formatMemoSnippet(
  memo: string | null | undefined,
  maxChars = 30,
): { text: string; truncated: boolean } {
  const chars = Array.from(memo ?? '')
  return {
    text: chars.slice(0, maxChars).join(''),
    truncated: chars.length > maxChars,
  }
}

export function resolveAddressDisplayName(item: {
  addressEntryDisplayName: string | null
  addressEntryArchived?: boolean | null
  addressSnapshot?: AddressPrintSnapshot | null
}): string {
  const snapshotName = item.addressSnapshot
    ? formatRecipientDisplayName(item.addressSnapshot).trim()
    : ''
  const name = snapshotName || item.addressEntryDisplayName?.trim() || ''
  if (name) {
    if (item.addressEntryArchived) {
      return `（アーカイブ済みの宛名）${name}`
    }
    return name
  }
  return '（削除済みの宛名）'
}

export function resolveSenderDisplayName(item: {
  senderEntryLabel: string | null
  senderEntryDisplayName: string | null
  senderEntryArchived?: boolean | null
  senderSnapshot?: SenderPrintSnapshot | null
}): string {
  const snapshotName = item.senderSnapshot
    ? formatSenderDisplayNameFromSnapshot(item.senderSnapshot).trim()
    : ''
  const label =
    snapshotName ||
    item.senderEntryLabel?.trim() ||
    item.senderEntryDisplayName?.trim() ||
    ''
  if (label) {
    if (item.senderEntryArchived) {
      return `（アーカイブ済み）${label}`
    }
    return label
  }
  return '（削除済みの差出人）'
}

export function resolveAddressPostalAndLine(item: {
  addressEntryAddressLine: string | null
  addressSnapshot?: AddressPrintSnapshot | null
}): { postalCode: string; addressLine: string } {
  if (item.addressSnapshot) {
    return {
      postalCode: formatPostalCode(item.addressSnapshot.postalCode),
      addressLine: formatSnapshotAddressLine(item.addressSnapshot) || '—',
    }
  }
  return {
    postalCode: '',
    addressLine: item.addressEntryAddressLine?.trim() || '—',
  }
}

export function resolveSenderPostalAndLine(item: {
  senderSnapshot?: SenderPrintSnapshot | null
}): { postalCode: string; addressLine: string } {
  if (item.senderSnapshot) {
    return {
      postalCode: formatPostalCode(item.senderSnapshot.postalCode),
      addressLine: formatSnapshotAddressLine(item.senderSnapshot) || '—',
    }
  }
  return { postalCode: '', addressLine: '—' }
}

export function fromPostcardSendDto(dto: PostcardSendDto): PostcardSendListItem {
  return {
    id: dto.id,
    printJobId: dto.print_job_id,
    addressEntryId: dto.address_entry_id,
    senderEntryId: dto.sender_entry_id,
    postcardType: dto.postcard_type === 'mochu' ? 'mochu' : 'nenga',
    sentOn: dto.sent_on,
    source: dto.source === 'manual' ? 'manual' : 'print',
    memo: dto.memo,
    addressEntryDisplayName: dto.address_entry_display_name,
    addressEntryArchived: dto.address_entry_archived,
    senderEntryLabel: dto.sender_entry_label,
    senderEntryDisplayName: dto.sender_entry_display_name,
    senderEntryArchived: dto.sender_entry_archived,
    addressSnapshot: parseAddressSnapshot(dto.address_snapshot),
    senderSnapshot: parseSenderSnapshot(dto.sender_snapshot),
  }
}

export function fromPostcardSendDtoToDetail(dto: PostcardSendDto): PostcardSendDetail {
  const base = fromPostcardSendDto(dto)
  const fromSnapshot = base.addressSnapshot
    ? formatSnapshotAddressLine(base.addressSnapshot)
    : ''
  return {
    ...base,
    createdAt: dto.created_at,
    updatedAt: dto.updated_at,
    addressEntryAddressLine: fromSnapshot || dto.address_entry_address_line,
  }
}

export function fromSendStatusItemDto(dto: SendStatusItemDto): SendStatusItem {
  return {
    addressEntryId: dto.address_entry_id,
    displayName: dto.display_name,
    addressSummary: dto.address_summary,
    lastSentOn: dto.last_sent_on,
    sendCount: dto.send_count,
  }
}

export function createInitialPostcardSendFormValues(): PostcardSendFormValues {
  return {
    sentOn: formatLocalDate(new Date()),
    postcardType: 'nenga',
    memo: '',
    addressEntryId: null,
    addressEntryDisplayName: null,
    senderEntryId: null,
    senderEntryLabel: null,
  }
}

export function formValuesFromDetail(detail: PostcardSendDetail): PostcardSendFormValues {
  const addressName = detail.addressSnapshot
    ? formatRecipientDisplayName(detail.addressSnapshot).trim()
    : ''
  const senderName = detail.senderSnapshot
    ? formatSenderDisplayNameFromSnapshot(detail.senderSnapshot).trim()
    : ''
  return {
    sentOn: detail.sentOn,
    postcardType: detail.postcardType,
    memo: detail.memo ?? '',
    addressEntryId: detail.addressEntryId,
    addressEntryDisplayName:
      addressName || detail.addressEntryDisplayName || detail.addressEntryId,
    senderEntryId: detail.senderEntryId,
    senderEntryLabel:
      detail.senderEntryLabel?.trim() ||
      senderName ||
      detail.senderEntryDisplayName ||
      detail.senderEntryId,
  }
}

/** 履歴タブ: 全期間 ∪ years ∪ 今年 */
export function buildHistoryYearOptions(
  availableYears: number[] = [],
  currentYear: number = currentLocalYear(),
): { value: string; label: string }[] {
  const years = new Set<number>(availableYears.filter((y) => Number.isFinite(y)))
  years.add(currentYear)
  const sorted = [...years].sort((a, b) => b - a)
  return [
    { value: '', label: '全期間' },
    ...sorted.map((y) => ({ value: String(y), label: String(y) })),
  ]
}

/** 送付状況タブ対象年: {今年, 今年-1} ∪ years ∪ 現在値（全期間なし） */
export function buildStatusYearOptions(
  availableYears: number[] = [],
  currentYear: number = currentLocalYear(),
  selectedYear?: number,
): { value: string; label: string }[] {
  const years = new Set<number>(availableYears.filter((y) => Number.isFinite(y)))
  years.add(currentYear)
  years.add(currentYear - 1)
  if (selectedYear != null && Number.isFinite(selectedYear)) {
    years.add(selectedYear)
  }
  const sorted = [...years].sort((a, b) => b - a)
  return sorted.map((y) => ({ value: String(y), label: String(y) }))
}

/** 受取年: {対象年, 対象年-1} ∪ receipt years ∪ 現在の receiptYear */
export function buildReceiptYearOptions(
  targetYear: number,
  availableReceiptYears: number[] = [],
  selectedReceiptYear?: number,
): { value: string; label: string }[] {
  const years = new Set<number>(availableReceiptYears.filter((y) => Number.isFinite(y)))
  years.add(targetYear)
  years.add(targetYear - 1)
  if (selectedReceiptYear != null && Number.isFinite(selectedReceiptYear)) {
    years.add(selectedReceiptYear)
  }
  const sorted = [...years].sort((a, b) => b - a)
  return sorted.map((y) => ({ value: String(y), label: String(y) }))
}
