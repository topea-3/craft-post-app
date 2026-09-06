import { toMm } from './layoutMath'

/** Scale applied to layoutSpec font sizes (design tweak). */
export const LAYOUT_FONT_SCALE = 1.5

/** Extra scale for sender address / name (relative to LAYOUT_FONT_SCALE). */
export const SENDER_CONTENT_FONT_SCALE = 0.8

/** Gap between 姓・名・敬称 (and co-recipient / sender name parts) in one column. */
export const NAME_PART_GAP_MM = 2

/** Address line2 starts this far below address line1 (same for recipient / sender). */
export const ADDRESS_LINE2_OFFSET_Y_MM = 10

/** Address line3 (building) starts this far below address line2. */
export const ADDRESS_LINE3_OFFSET_Y_MM = 10

/** Horizontal gap between consecutive sender address columns (mm). */
export const SENDER_ADDRESS_COL_GAP_MM = 4

/** Middle-dot used to join co-recipients in one vertical column (no line break). */
export const CO_RECIPIENT_JOIN_MARK = '・'

/**
 * 宛名郵便番号: 官製はがき上部の実線枠に合わせたスロット（mm）。
 * slotMm = 桁ボックス中心間隔。groupExtraMm = 3桁目と4桁目の追加すき間。
 */
export const RECIPIENT_POSTAL_LAYOUT = {
  originMm: { x: 44.2, y: 15.2 }, // 左へ4mm・下へ3mm
  slotMm: 6.85, // 文字間 +1mm
  groupExtraMm: -0.6, // 上下桁間 -3mm（2.4 → -0.6）
  fontSizePt: 11,
} as const

/**
 * 差出人郵便番号: 左下の点線枠に合わせたスロット（mm）。
 */
export const SENDER_POSTAL_LAYOUT = {
  originMm: { x: 6.2, y: 121.5 }, // 下へ5mm
  slotMm: 4.15,
  groupExtraMm: -0.4, // 上下桁間 -2mm（1.6 → -0.4）
  fontSizePt: 8,
} as const

/** @deprecated kept for tests — prefer slot layout above */
export const POSTAL_DIGIT_GAP_MM = 2
export const RECIPIENT_POSTAL_GROUP_GAP_MM = 4
export const SENDER_POSTAL_GROUP_GAP_MM = 2

const VERTICAL_DASH_CHARS = new Set([
  '-',
  'ー', // katakana-hiragana prolonged sound mark
  '－', // fullwidth hyphen-minus (common in JP addresses)
  '−', // minus sign
  '‐',
  '‒',
  '–',
  '—',
  'ｰ', // halfwidth prolonged sound mark
  '―',
  '─',
  '━',
])

export function isVerticalDashChar(ch: string): boolean {
  return VERTICAL_DASH_CHARS.has(ch)
}

/** Approximate vertical advance of one full-width glyph in mm. */
export function verticalCharAdvanceMm(fontSizePt: number): number {
  return toMm(fontSizePt)
}

export function verticalTextAdvanceMm(text: string, fontSizePt: number): number {
  if (!text) return 0
  return [...text].length * verticalCharAdvanceMm(fontSizePt)
}

export type PostalDigitGroups = {
  upper: string[]
  lower: string[]
}

/** Strip formatting and split into 3+4 digit groups when possible. */
export function splitPostalDigits(postalCode: string): PostalDigitGroups {
  const digits = postalCode.replace(/\D/g, '').slice(0, 7)
  return {
    upper: [...digits.slice(0, 3)],
    lower: [...digits.slice(3)],
  }
}

/** Left offset (mm) of digit index 0..6 within a postal slot row. */
export function postalDigitOffsetXMm(
  index: number,
  slotMm: number,
  groupExtraMm: number,
): number {
  const groupGap = index >= 3 ? groupExtraMm : 0
  return index * slotMm + groupGap
}

export function scaledFontSizePt(basePt: number): number {
  return basePt * LAYOUT_FONT_SCALE
}

export function scaledSenderContentFontSizePt(basePt: number): number {
  return basePt * LAYOUT_FONT_SCALE * SENDER_CONTENT_FONT_SCALE
}

export function isSenderContentLayer(layerId: string): boolean {
  return (
    layerId.startsWith('sender.address') ||
    layerId.startsWith('sender.primary') ||
    layerId.startsWith('sender.co')
  )
}

export type CoJoinLayer = { id: string; text: string; hiddenByRule: boolean }

/**
 * 連名を改行せず「・」でつなぐ。マークは直前の可視要素の text 末尾に付与する。
 * partsInPersonOrder: 1人分の描画順（姓→名→敬称）。
 */
export function appendCoRecipientJoinMarks<T extends CoJoinLayer>(
  layers: T[],
  personPartIds: string[][],
): T[] {
  const presentPersonIndexes: number[] = []
  for (let i = 0; i < personPartIds.length; i++) {
    const hasVisible = personPartIds[i].some((id) => {
      const layer = layers.find((l) => l.id === id)
      return Boolean(layer && !layer.hiddenByRule && layer.text)
    })
    if (hasVisible) presentPersonIndexes.push(i)
  }

  if (presentPersonIndexes.length < 2) return layers

  const next = layers.map((l) => ({ ...l }))
  for (let p = 0; p < presentPersonIndexes.length - 1; p++) {
    const partIds = personPartIds[presentPersonIndexes[p]]
    let target: T | undefined
    for (const id of partIds) {
      const layer = next.find((l) => l.id === id)
      if (layer && !layer.hiddenByRule && layer.text) {
        target = layer
      }
    }
    if (target && !target.text.endsWith(CO_RECIPIENT_JOIN_MARK)) {
      target.text = `${target.text}${CO_RECIPIENT_JOIN_MARK}`
    }
  }
  return next
}
