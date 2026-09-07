import type { PrintLayerId } from './layout/printLayers'
import { ALL_PRINT_LAYER_IDS } from './layout/printLayers'

export type PostcardType = 'nenga' | 'mochu'

export type CoRecipientPrint = {
  last: string
  first: string
  omitLast: boolean
}

export type AddressPrintSnapshot = {
  addressEntryId: string
  postalCode: string
  addressLine1: string
  addressLine2: string
  addressLine3: string
  primaryLast: string
  primaryFirst: string
  coRecipients: CoRecipientPrint[]
  honorificPrint: string
}

export type SenderPrintSnapshot = {
  senderEntryId: string
  postalCode: string
  addressLine1: string
  addressLine2: string
  addressLine3: string
  primaryLast: string
  primaryFirst: string
  coRecipients: CoRecipientPrint[]
}

export type LayerVisibility = Record<PrintLayerId, boolean>

export type PrintJobItem = {
  address: AddressPrintSnapshot
  sender: SenderPrintSnapshot
  layerVisibility: LayerVisibility
}

export type ExcludedAlert = {
  addressEntryId: string
  reason: string
  displayName?: string
}

export type PrintJobDraft = {
  addressEntryIds: string[]
  excludedAlerts?: ExcludedAlert[]
}

export type LayoutOffset = { dx: number; dy: number }
export type LayoutOffsets = Record<PrintLayerId, LayoutOffset>

export type PrintLayoutPreference = {
  layerId: PrintLayerId
  offsetXPt: number
  offsetYPt: number
}

/** Backend DTOs (snake_case) */

export type CoRecipientPrintDto = {
  last: string
  first: string
  omit_last: boolean
}

export type AddressPrintSnapshotDto = {
  address_entry_id: string
  postal_code: string
  address_line1: string
  address_line2: string
  address_line3?: string
  primary_last: string
  primary_first: string
  co_recipients: CoRecipientPrintDto[]
  honorific_print: string
}

export type SenderPrintSnapshotDto = {
  sender_entry_id: string
  postal_code: string
  address_line1: string
  address_line2: string
  address_line3?: string
  primary_last: string
  primary_first: string
  co_recipients: CoRecipientPrintDto[]
}

export type PrintJobItemDto = {
  address: AddressPrintSnapshotDto
  sender: SenderPrintSnapshotDto
}

export type ExcludedAlertDto = {
  address_entry_id: string
  reason: string
  display_name?: string | null
}

export type ResolvePrintJobItemsDto = {
  items: PrintJobItemDto[]
  excluded?: ExcludedAlertDto[]
  excluded_alerts?: ExcludedAlertDto[]
}

export type PrintLayoutPreferenceDto = {
  layer_id: string
  offset_x_pt: number
  offset_y_pt: number
}

export type AddressEntriesInvalidError = {
  code: 'ADDRESS_ENTRIES_INVALID'
  entries: { address_entry_id: string; reason: string }[]
}

export const POSTCARD_TYPE_OPTIONS: { value: PostcardType; label: string }[] = [
  { value: 'nenga', label: '年賀状' },
  { value: 'mochu', label: '喪中はがき' },
]

export const MAX_PRINT_SELECTION = 200

export function createDefaultLayerVisibility(): LayerVisibility {
  const visibility = {} as LayerVisibility
  for (const id of ALL_PRINT_LAYER_IDS) {
    visibility[id] = true
  }
  return visibility
}

export function createDefaultLayoutOffsets(): LayoutOffsets {
  const offsets = {} as LayoutOffsets
  for (const id of ALL_PRINT_LAYER_IDS) {
    offsets[id] = { dx: 0, dy: 0 }
  }
  return offsets
}

export function fromCoRecipientPrintDto(dto: CoRecipientPrintDto): CoRecipientPrint {
  return {
    last: dto.last,
    first: dto.first,
    omitLast: dto.omit_last,
  }
}

export function toCoRecipientPrintDto(value: CoRecipientPrint): CoRecipientPrintDto {
  return {
    last: value.last,
    first: value.first,
    omit_last: value.omitLast,
  }
}

export function fromAddressPrintSnapshotDto(dto: AddressPrintSnapshotDto): AddressPrintSnapshot {
  return {
    addressEntryId: dto.address_entry_id,
    postalCode: dto.postal_code,
    addressLine1: dto.address_line1,
    addressLine2: dto.address_line2 ?? '',
    addressLine3: dto.address_line3 ?? '',
    primaryLast: dto.primary_last,
    primaryFirst: dto.primary_first,
    coRecipients: (dto.co_recipients ?? []).map(fromCoRecipientPrintDto),
    honorificPrint: dto.honorific_print ?? '',
  }
}

export function toAddressPrintSnapshotDto(value: AddressPrintSnapshot): AddressPrintSnapshotDto {
  return {
    address_entry_id: value.addressEntryId,
    postal_code: value.postalCode,
    address_line1: value.addressLine1,
    address_line2: value.addressLine2,
    address_line3: value.addressLine3,
    primary_last: value.primaryLast,
    primary_first: value.primaryFirst,
    co_recipients: value.coRecipients.map(toCoRecipientPrintDto),
    honorific_print: value.honorificPrint,
  }
}

export function fromSenderPrintSnapshotDto(dto: SenderPrintSnapshotDto): SenderPrintSnapshot {
  return {
    senderEntryId: dto.sender_entry_id,
    postalCode: dto.postal_code,
    addressLine1: dto.address_line1,
    addressLine2: dto.address_line2 ?? '',
    addressLine3: dto.address_line3 ?? '',
    primaryLast: dto.primary_last,
    primaryFirst: dto.primary_first,
    coRecipients: (dto.co_recipients ?? []).map(fromCoRecipientPrintDto),
  }
}

export function toSenderPrintSnapshotDto(value: SenderPrintSnapshot): SenderPrintSnapshotDto {
  return {
    sender_entry_id: value.senderEntryId,
    postal_code: value.postalCode,
    address_line1: value.addressLine1,
    address_line2: value.addressLine2,
    address_line3: value.addressLine3,
    primary_last: value.primaryLast,
    primary_first: value.primaryFirst,
    co_recipients: value.coRecipients.map(toCoRecipientPrintDto),
  }
}

export function fromPrintJobItemDto(dto: PrintJobItemDto): PrintJobItem {
  return {
    address: fromAddressPrintSnapshotDto(dto.address),
    sender: fromSenderPrintSnapshotDto(dto.sender),
    layerVisibility: createDefaultLayerVisibility(),
  }
}

export function fromExcludedAlertDto(dto: ExcludedAlertDto): ExcludedAlert {
  const displayName = dto.display_name?.trim()
  return {
    addressEntryId: dto.address_entry_id,
    reason: dto.reason,
    displayName: displayName ? displayName : undefined,
  }
}

export function excludedAlertLabel(alert: ExcludedAlert): string {
  return alert.displayName?.trim() || alert.addressEntryId
}

export function fromResolvePrintJobItemsDto(dto: ResolvePrintJobItemsDto): {
  items: PrintJobItem[]
  excludedAlerts: ExcludedAlert[]
} {
  const excludedRaw = dto.excluded_alerts ?? dto.excluded ?? []
  return {
    items: (dto.items ?? []).map(fromPrintJobItemDto),
    excludedAlerts: excludedRaw.map(fromExcludedAlertDto),
  }
}

export function fromPrintLayoutPreferenceDto(
  dto: PrintLayoutPreferenceDto,
): PrintLayoutPreference | null {
  const layerId = dto.layer_id as PrintLayerId
  if (!ALL_PRINT_LAYER_IDS.includes(layerId)) return null
  return {
    layerId,
    offsetXPt: dto.offset_x_pt,
    offsetYPt: dto.offset_y_pt,
  }
}

export function formatRecipientDisplayName(snapshot: AddressPrintSnapshot): string {
  const primary = `${snapshot.primaryLast} ${snapshot.primaryFirst}`.trim()
  const honorific = snapshot.honorificPrint.trim()
  const base = honorific ? `${primary} ${honorific}` : primary
  if (snapshot.coRecipients.length === 0) return base
  const co = snapshot.coRecipients
    .map((c) => {
      if (c.omitLast) return c.first
      return `${c.last} ${c.first}`.trim()
    })
    .filter(Boolean)
    .join('・')
  return co ? `${base}・${co}` : base
}

export function formatSenderDisplayNameFromSnapshot(snapshot: SenderPrintSnapshot): string {
  const primary = `${snapshot.primaryLast} ${snapshot.primaryFirst}`.trim()
  if (snapshot.coRecipients.length === 0) return primary
  const co = snapshot.coRecipients
    .map((c) => {
      if (c.omitLast) return c.first
      return `${c.last} ${c.first}`.trim()
    })
    .filter(Boolean)
    .join('・')
  return co ? `${primary}・${co}` : primary
}

export function excludedReasonLabel(reason: string): string {
  switch (reason) {
    case 'no_sender_link':
    case 'unlinked':
      return '差出人未紐づけ'
    case 'sender_archived':
      return '差出人がアーカイブ済み'
    case 'archived':
      return 'アーカイブ済み'
    case 'not_found':
      return '見つかりません'
    default:
      return reason
  }
}

/**
 * Parse Validation JSON from resolve_print_job_items.
 * Returns null if not the structured ADDRESS_ENTRIES_INVALID payload.
 */
export function parseAddressEntriesInvalidError(
  error: unknown,
): AddressEntriesInvalidError | null {
  const raw = typeof error === 'string' ? error : error instanceof Error ? error.message : null
  if (!raw) return null
  try {
    const parsed = JSON.parse(raw) as Partial<AddressEntriesInvalidError>
    if (parsed?.code !== 'ADDRESS_ENTRIES_INVALID' || !Array.isArray(parsed.entries)) {
      return null
    }
    return {
      code: 'ADDRESS_ENTRIES_INVALID',
      entries: parsed.entries.map((e) => ({
        address_entry_id: String(e.address_entry_id ?? ''),
        reason: String(e.reason ?? ''),
      })),
    }
  } catch {
    return null
  }
}

export function invokeErrorMessage(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}
