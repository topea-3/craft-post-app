import { toMm } from './layoutMath'
import type { PointMm } from './layoutTypes'
import type { PrintLayerId } from './printLayers'
import { verticalCharAdvanceMm } from './printTypography'

/** 印刷方針の下限。これより小さくしない。 */
export const MIN_RECIPIENT_ADDRESS_FONT_PT = 6

/** 枠に収まらないときに下げる刻み。 */
export const ADDRESS_FONT_STEP_PT = 0.5

/** 列ピッチを詰めるときの字形どうしの最小すき間。 */
export const ADDRESS_COLUMN_MIN_GAP_MM = 0.4

export type RecipientAddressField = {
  id: PrintLayerId
  text: string
  shown: boolean
  /** ユーザーオフセット適用後の列原点（top-left、mm）。 */
  anchorMm: PointMm
}

export type AddressColumnPlacement = {
  text: string
  xMm: number
  yMm: number
}

export type RecipientAddressLayout = {
  fontSizePt: number
  columnPitchMm: number
  columns: Partial<Record<PrintLayerId, AddressColumnPlacement[]>>
  /** 宛名氏名・連名列の X から引く量（mm。正で左へ）。 */
  nameShiftLeftMm: number
}

/**
 * 縦書き 1 列に入る文字数。下端ちょうどまでは含め、はみ出す文字は次列。
 * 高さが無いときは 1（無限ループを避ける）。
 */
export function maxVerticalChars(startYMm: number, fontSizePt: number, bottomYMm: number): number {
  const advance = verticalCharAdvanceMm(fontSizePt)
  const available = bottomYMm - startYMm
  if (!(advance > 0) || !(available > 0)) return 1
  return Math.max(1, Math.floor(available / advance + 1e-8))
}

/** 右列から順に maxChars 字ずつ分割する。コードポイント単位。 */
export function splitVerticalColumns(text: string, maxChars: number): string[] {
  const chars = [...text]
  if (chars.length === 0) return []
  const size = Math.max(1, Math.floor(maxChars))
  const columns: string[] = []
  for (let i = 0; i < chars.length; i += size) {
    columns.push(chars.slice(i, i + size).join(''))
  }
  return columns
}

type PlacedColumns = {
  columns: Partial<Record<PrintLayerId, AddressColumnPlacement[]>>
  nameShiftLeftMm: number
  leftmostMm: number
}

function isActive(field: RecipientAddressField): boolean {
  return field.shown && field.text.length > 0
}

function fitsLeft(leftmostMm: number, leftLimitMm: number): boolean {
  return leftmostMm >= leftLimitMm - 1e-4
}

function placeColumns(
  fields: RecipientAddressField[],
  fontSizePt: number,
  pitchMm: number,
  bottomYMm: number,
): PlacedColumns {
  const columns: Partial<Record<PrintLayerId, AddressColumnPlacement[]>> = {}
  let extraBefore = 0
  // この計算で左へ動かした端だけを見る。アンカーそのものが余白外でも縮小しない。
  let leftmost = Number.POSITIVE_INFINITY

  for (const field of fields) {
    if (!isActive(field)) continue
    const parts = splitVerticalColumns(
      field.text,
      maxVerticalChars(field.anchorMm.y, fontSizePt, bottomYMm),
    )
    const placed: AddressColumnPlacement[] = []
    for (let i = 0; i < parts.length; i++) {
      const xMm = field.anchorMm.x - (extraBefore + i) * pitchMm
      if (extraBefore + i > 0) leftmost = Math.min(leftmost, xMm)
      placed.push({ text: parts[i], xMm, yMm: field.anchorMm.y })
    }
    columns[field.id] = placed
    extraBefore += Math.max(0, parts.length - 1)
  }

  return { columns, nameShiftLeftMm: extraBefore * pitchMm, leftmostMm: leftmost }
}

/**
 * 指定フォントでの列数はピッチに依存しない。
 * 左端が leftLimit 以上になる最大ピッチ（spec 以下）を返す。負になり得る。
 */
function maxPitchWithinLeftLimit(
  fields: RecipientAddressField[],
  fontSizePt: number,
  bottomYMm: number,
  leftLimitMm: number,
  pitchBase: number,
): number {
  let extraBefore = 0
  let limit = pitchBase
  for (const field of fields) {
    if (!isActive(field)) continue
    const parts = splitVerticalColumns(
      field.text,
      maxVerticalChars(field.anchorMm.y, fontSizePt, bottomYMm),
    )
    for (let i = 0; i < parts.length; i++) {
      const steps = extraBefore + i
      if (steps > 0) {
        limit = Math.min(limit, (field.anchorMm.x - leftLimitMm) / steps)
      }
    }
    extraBefore += Math.max(0, parts.length - 1)
  }
  return limit
}

function joinColumns(columns: AddressColumnPlacement[] | undefined): string {
  return (columns ?? []).map((column) => column.text).join('')
}

/**
 * 宛名住所 1〜3 を下端で折り返し、後続列と氏名を左へずらす。
 * フィールドは右から（住所1 → 住所2 → 住所3）の順。
 */
export function layoutRecipientAddressColumns(input: {
  fields: RecipientAddressField[]
  specPitchMm: number
  bottomYMm: number
  leftLimitMm: number
  /** 呼び出し側の氏名左端。余白判定には使わない（氏名だけのはみ出しでは縮小しない）。 */
  nameLeftMm: number | null
  initialFontSizePt: number
}): RecipientAddressLayout {
  const { fields, specPitchMm, bottomYMm, leftLimitMm, initialFontSizePt } = input
  const fallbackPitch =
    verticalCharAdvanceMm(initialFontSizePt) + ADDRESS_COLUMN_MIN_GAP_MM
  const pitchBase = specPitchMm > 0 ? specPitchMm : fallbackPitch

  let fontSizePt = initialFontSizePt
  let columnPitchMm = pitchBase
  let placed = placeColumns(fields, fontSizePt, columnPitchMm, bottomYMm)

  while (!fitsLeft(placed.leftmostMm, leftLimitMm) && fontSizePt > MIN_RECIPIENT_ADDRESS_FONT_PT + 1e-6) {
    const next = Math.round((fontSizePt - ADDRESS_FONT_STEP_PT) * 10) / 10
    fontSizePt = Math.max(MIN_RECIPIENT_ADDRESS_FONT_PT, next)
    placed = placeColumns(fields, fontSizePt, columnPitchMm, bottomYMm)
  }

  if (!fitsLeft(placed.leftmostMm, leftLimitMm)) {
    const minPitch = toMm(fontSizePt) + ADDRESS_COLUMN_MIN_GAP_MM
    const limitPitch = maxPitchWithinLeftLimit(
      fields,
      fontSizePt,
      bottomYMm,
      leftLimitMm,
      pitchBase,
    )
    const compressed = Math.min(pitchBase, Math.max(minPitch, limitPitch))
    if (compressed < columnPitchMm - 1e-6) {
      columnPitchMm = compressed
      placed = placeColumns(fields, fontSizePt, columnPitchMm, bottomYMm)
    }
  }

  return {
    fontSizePt,
    columnPitchMm,
    columns: placed.columns,
    nameShiftLeftMm: placed.nameShiftLeftMm,
  }
}

/** テスト用。配置した列を連結すると元の文字列になる。 */
export function placedAddressText(
  layout: RecipientAddressLayout,
  id: PrintLayerId,
): string {
  return joinColumns(layout.columns[id])
}
