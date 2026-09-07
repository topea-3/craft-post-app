import { invoke } from '@tauri-apps/api/core'
import type {
  AddressPrintSnapshot,
  AddressPrintSnapshotDto,
  PostcardType,
  PrintJobItem,
  PrintLayoutPreference,
  PrintLayoutPreferenceDto,
  ResolvePrintJobItemsDto,
  SenderPrintSnapshot,
  SenderPrintSnapshotDto,
} from './types'
import {
  fromAddressPrintSnapshotDto,
  fromPrintLayoutPreferenceDto,
  fromResolvePrintJobItemsDto,
  fromSenderPrintSnapshotDto,
  toAddressPrintSnapshotDto,
  toSenderPrintSnapshotDto,
} from './types'
import type { LayoutOffsets } from './types'
import type { PrintLayerId } from './layout/printLayers'

/**
 * Tauri command 引数のトップレベルキーは camelCase（Rust snake_case が自動変換される）。
 * ネストした DTO フィールドは serde 既定どおり snake_case のまま。
 */
export async function filterActiveAddressEntryIds(ids: string[]): Promise<string[]> {
  const addressEntryIds = ids.slice(0, 200)
  return invoke<string[]>('filter_active_address_entry_ids', { addressEntryIds })
}

export async function resolvePrintJobItems(ids: string[]): Promise<{
  items: PrintJobItem[]
  excludedAlerts: { addressEntryId: string; reason: string }[]
}> {
  const addressEntryIds = ids.slice(0, 200)
  const dto = await invoke<ResolvePrintJobItemsDto>('resolve_print_job_items', {
    addressEntryIds,
  })
  return fromResolvePrintJobItemsDto(dto)
}

export async function buildAddressPrintSnapshot(
  addressEntryId: string,
): Promise<AddressPrintSnapshot> {
  const dto = await invoke<AddressPrintSnapshotDto>('build_address_print_snapshot', {
    addressEntryId,
  })
  return fromAddressPrintSnapshotDto(dto)
}

export async function buildSenderPrintSnapshot(
  senderEntryId: string,
): Promise<SenderPrintSnapshot> {
  const dto = await invoke<SenderPrintSnapshotDto>('build_sender_print_snapshot', {
    senderEntryId,
  })
  return fromSenderPrintSnapshotDto(dto)
}

export async function listPrintLayoutPreferences(
  postcardType: PostcardType,
): Promise<PrintLayoutPreference[]> {
  const list = await invoke<PrintLayoutPreferenceDto[]>('list_print_layout_preferences', {
    postcardType,
  })
  return (list ?? [])
    .map(fromPrintLayoutPreferenceDto)
    .filter((p): p is PrintLayoutPreference => p !== null)
}

export async function savePrintLayoutPreferences(
  postcardType: PostcardType,
  offsets: LayoutOffsets,
): Promise<void> {
  const offsetDtos = (Object.entries(offsets) as [PrintLayerId, { dx: number; dy: number }][]).map(
    ([layerId, offset]) => ({
      layer_id: layerId,
      offset_x_pt: offset.dx,
      offset_y_pt: offset.dy,
    }),
  )
  await invoke('save_print_layout_preferences', {
    postcardType,
    offsets: offsetDtos,
  })
}

export type PostcardSendBatchItemInput = {
  address: AddressPrintSnapshot
  sender: SenderPrintSnapshot
}

export async function createPostcardSendsBatch(params: {
  printJobId: string
  postcardType: PostcardType
  items: PostcardSendBatchItemInput[]
}): Promise<void> {
  await invoke('create_postcard_sends_batch', {
    input: {
      print_job_id: params.printJobId,
      postcard_type: params.postcardType,
      items: params.items.map((item) => ({
        address_entry_id: item.address.addressEntryId,
        sender_entry_id: item.sender.senderEntryId,
        address_snapshot: toAddressPrintSnapshotDto(item.address),
        sender_snapshot: toSenderPrintSnapshotDto(item.sender),
      })),
    },
  })
}

/** Re-snapshot all items (all-or-nothing). Throws on any failure. */
export async function resnapshotPrintJobItems(
  items: PrintJobItem[],
): Promise<PrintJobItem[]> {
  const next: PrintJobItem[] = []
  for (const item of items) {
    const address = await buildAddressPrintSnapshot(item.address.addressEntryId)
    const sender = await buildSenderPrintSnapshot(item.sender.senderEntryId)
    next.push({
      address,
      sender,
      layerVisibility: item.layerVisibility,
    })
  }
  return next
}
