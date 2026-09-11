/** printJobDraft を作業選択と分離して置換する（確認 OK 時のみ呼ぶ） */
export function replacePrintJobDraftAddressIds(addressEntryIds: string[]): void {
  const MAX = 200
  const seen = new Set<string>()
  const ids: string[] = []
  for (const id of addressEntryIds) {
    if (seen.has(id)) continue
    seen.add(id)
    ids.push(id)
    if (ids.length >= MAX) break
  }
  sessionStorage.setItem(
    'printJobDraft',
    JSON.stringify({ addressEntryIds: ids, excludedAlerts: [] }),
  )
}

const SEND_STATUS_SELECTED_IDS_KEY = 'sendStatusSelectedIds'

/** SND005 作業選択（printJobDraft とは分離） */
export function readSendStatusSelectedIds(): string[] {
  try {
    const raw = sessionStorage.getItem(SEND_STATUS_SELECTED_IDS_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed.filter((id): id is string => typeof id === 'string')
  } catch {
    return []
  }
}

export function writeSendStatusSelectedIds(ids: string[]): void {
  const MAX = 200
  const seen = new Set<string>()
  const next: string[] = []
  for (const id of ids) {
    if (seen.has(id)) continue
    seen.add(id)
    next.push(id)
    if (next.length >= MAX) break
  }
  sessionStorage.setItem(SEND_STATUS_SELECTED_IDS_KEY, JSON.stringify(next))
}

export function clearSendStatusSelectedIds(): void {
  sessionStorage.removeItem(SEND_STATUS_SELECTED_IDS_KEY)
}

export function readPrintJobDraftAddressIds(): string[] {
  try {
    const raw = sessionStorage.getItem('printJobDraft')
    if (!raw) return []
    const parsed = JSON.parse(raw) as { addressEntryIds?: unknown }
    if (!Array.isArray(parsed.addressEntryIds)) return []
    return parsed.addressEntryIds.filter((id): id is string => typeof id === 'string')
  } catch {
    return []
  }
}

export function syncPrintPostcardType(postcardType: 'nenga' | 'mochu'): void {
  sessionStorage.setItem('printPostcardType', postcardType)
}
