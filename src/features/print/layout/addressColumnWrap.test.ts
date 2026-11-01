import { describe, expect, it } from 'vitest'
import { mochuLayoutSpec } from './mochuLayoutSpec'
import { nengaLayoutSpec } from './nengaLayoutSpec'
import {
  scaledFontSizePt,
  scaledSenderContentFontSizePt,
  SENDER_ADDRESS_POSTAL_CLEARANCE_MM,
  SENDER_POSTAL_LAYOUT,
  verticalCharAdvanceMm,
} from './printTypography'
import {
  ADDRESS_COLUMN_MIN_GAP_MM,
  MIN_RECIPIENT_ADDRESS_FONT_PT,
  layoutRecipientAddressColumns,
  maxVerticalChars,
  placedAddressText,
  splitVerticalColumns,
  type RecipientAddressField,
} from './addressColumnWrap'
import type { PrintLayerId } from './printLayers'

const spec = nengaLayoutSpec
const pitch =
  spec.layers['recipient.address1'].originMm.x - spec.layers['recipient.address2'].originMm.x
const bottom = spec.recipientAddressBottomMm
const font = scaledFontSizePt(spec.layers['recipient.address1'].fontSizePt)
const nameLeft = spec.layers['recipient.coLast.1'].originMm.x

function field(
  id: PrintLayerId,
  text: string,
  shown = true,
  anchor = spec.layers[id].originMm,
): RecipientAddressField {
  return { id, text, shown, anchorMm: { ...anchor } }
}

function layout(
  fields: RecipientAddressField[],
  overrides: Partial<Parameters<typeof layoutRecipientAddressColumns>[0]> = {},
) {
  return layoutRecipientAddressColumns({
    fields,
    specPitchMm: pitch,
    bottomYMm: bottom,
    leftLimitMm: spec.marginMm,
    nameLeftMm: nameLeft,
    initialFontSizePt: font,
    ...overrides,
  })
}

describe('maxVerticalChars', () => {
  it('stops the nenga address column above the lower design band', () => {
    const cardBottom = spec.postcard.height - spec.marginMm
    expect(spec.recipientAddressBottomMm).toBeLessThan(cardBottom)
    expect(mochuLayoutSpec.recipientAddressBottomMm).toBe(cardBottom)
    const capacity = maxVerticalChars(
      spec.layers['recipient.address1'].originMm.y,
      font,
      spec.recipientAddressBottomMm,
    )
    const toCardEdge = maxVerticalChars(
      spec.layers['recipient.address1'].originMm.y,
      font,
      cardBottom,
    )
    expect(capacity).toBeLessThan(toCardEdge)
    expect(capacity).toBe(17)
  })

  it('returns 1 when the anchor is already at the bottom', () => {
    expect(maxVerticalChars(bottom, font, bottom)).toBe(1)
    expect(maxVerticalChars(bottom + 10, font, bottom)).toBe(1)
  })
})

describe('splitVerticalColumns', () => {
  it('keeps code points and order', () => {
    expect(splitVerticalColumns('あいうえお', 2)).toEqual(['あい', 'うえ', 'お'])
  })

  it('returns empty for empty text', () => {
    expect(splitVerticalColumns('', 4)).toEqual([])
  })
})

describe('layoutRecipientAddressColumns', () => {
  it('does not move a short address or the name', () => {
    const result = layout([
      field('recipient.address1', '東京都千代田区'),
      field('recipient.address2', '神南1-1-1'),
      field('recipient.address3', 'ビル3F'),
    ])

    expect(result.fontSizePt).toBeCloseTo(font)
    expect(result.columnPitchMm).toBeCloseTo(pitch)
    expect(result.nameShiftLeftMm).toBe(0)
    expect(result.columns['recipient.address1']).toEqual([
      {
        text: '東京都千代田区',
        xMm: spec.layers['recipient.address1'].originMm.x,
        yMm: spec.layers['recipient.address1'].originMm.y,
      },
    ])
    expect(result.columns['recipient.address3']?.[0].xMm).toBeCloseTo(
      spec.layers['recipient.address3'].originMm.x,
    )
  })

  it('wraps sooner when the line starts lower on the card', () => {
    const text = 'あ'.repeat(
      maxVerticalChars(spec.layers['recipient.address1'].originMm.y, font, bottom),
    )
    const anchorX = spec.layers['recipient.address1'].originMm.x
    const high = layout([
      field('recipient.address1', text, true, { x: anchorX, y: 26 }),
      field('recipient.address2', '', false),
      field('recipient.address3', '', false),
    ])
    const low = layout([
      field('recipient.address1', text, true, { x: anchorX, y: 80 }),
      field('recipient.address2', '', false),
      field('recipient.address3', '', false),
    ])

    expect(high.columns['recipient.address1']).toHaveLength(1)
    expect(low.columns['recipient.address1']?.length ?? 0).toBeGreaterThan(1)
    expect(low.columns['recipient.address1']?.[0].yMm).toBe(80)
  })

  it('wraps overflow onto a left column and shifts the following address and name', () => {
    const capacity = maxVerticalChars(spec.layers['recipient.address1'].originMm.y, font, bottom)
    const text = 'あ'.repeat(capacity + 1)
    const result = layout([
      field('recipient.address1', text),
      field('recipient.address2', '神南1-1-1'),
      field('recipient.address3', '', false),
    ])

    const line1 = result.columns['recipient.address1']
    expect(line1).toHaveLength(2)
    expect(line1?.[0].text).toHaveLength(capacity)
    expect(line1?.[1].text).toBe('あ')
    expect(line1?.[1].xMm).toBeCloseTo(line1![0].xMm - pitch)
    expect(line1?.[0].yMm).toBeCloseTo(line1![1].yMm)
    expect(placedAddressText(result, 'recipient.address1')).toBe(text)
    expect(result.columns['recipient.address2']?.[0].xMm).toBeCloseTo(
      spec.layers['recipient.address2'].originMm.x - pitch,
    )
    expect(result.nameShiftLeftMm).toBeCloseTo(pitch)
    expect(result.fontSizePt).toBeCloseTo(font)
    expect(result.columns['recipient.address3']).toBeUndefined()
  })

  it('shifts only by earlier extra columns when a later line wraps', () => {
    const line3 = '建'.repeat(17)
    const result = layout([
      field('recipient.address1', '東京都千代田区'),
      field('recipient.address2', '神南1-1-1'),
      field('recipient.address3', line3),
    ])

    expect(result.columns['recipient.address1']).toHaveLength(1)
    expect(result.columns['recipient.address1']?.[0].xMm).toBeCloseTo(
      spec.layers['recipient.address1'].originMm.x,
    )
    expect(result.columns['recipient.address2']?.[0].xMm).toBeCloseTo(
      spec.layers['recipient.address2'].originMm.x,
    )
    expect(result.columns['recipient.address3']).toHaveLength(2)
    expect(result.nameShiftLeftMm).toBeCloseTo(pitch)
  })

  it('keeps a hidden middle line from consuming a column', () => {
    const result = layout([
      field('recipient.address1', 'あ'.repeat(21)),
      field('recipient.address2', '町名', false),
      field('recipient.address3', '建物'),
    ])

    expect(result.columns['recipient.address2']).toBeUndefined()
    expect(result.columns['recipient.address3']?.[0].xMm).toBeCloseTo(
      spec.layers['recipient.address3'].originMm.x - pitch,
    )
  })

  it('keeps a full-size wrap when only the sender co-name crosses the margin', () => {
    const building = 'パークシティ渋谷ザタワー301'
    const senderFont = scaledSenderContentFontSizePt(spec.layers['sender.address3'].fontSizePt)
    const bottomYMm = Math.min(
      spec.recipientAddressBottomMm,
      SENDER_POSTAL_LAYOUT.originMm.y - SENDER_ADDRESS_POSTAL_CLEARANCE_MM,
    )
    const result = layout(
      [
        field('sender.address1', '茨城県水戸市', true, spec.layers['sender.address1'].originMm),
        field('sender.address2', '差出2-1-1', true, spec.layers['sender.address2'].originMm),
        field('sender.address3', building, true, spec.layers['sender.address3'].originMm),
      ],
      {
        specPitchMm:
          spec.layers['sender.address1'].originMm.x - spec.layers['sender.address2'].originMm.x,
        bottomYMm,
        nameLeftMm: spec.layers['sender.coLast.1'].originMm.x,
        initialFontSizePt: senderFont,
      },
    )

    expect([...building]).toHaveLength(15)
    expect(result.fontSizePt).toBeCloseTo(senderFont)
    expect(result.columns['sender.address3']).toHaveLength(2)
    expect(result.nameShiftLeftMm).toBeCloseTo(
      spec.layers['sender.address1'].originMm.x - spec.layers['sender.address2'].originMm.x,
    )
    expect(placedAddressText(result, 'sender.address3')).toBe(building)
  })

  it('uses one column on mochu and two on nenga for an 18 character address', () => {
    const text = 'あ'.repeat(18)
    expect(layout([field('recipient.address1', text)]).columns['recipient.address1']).toHaveLength(
      2,
    )
    expect(
      layout([field('recipient.address1', text)], {
        bottomYMm: mochuLayoutSpec.recipientAddressBottomMm,
      }).columns['recipient.address1'],
    ).toHaveLength(1)
  })

  it('compresses column pitch at the minimum font when address columns cannot fit', () => {
    const text = 'あ'.repeat(10)
    const leftLimitMm = 5
    const anchorX = 20
    const result = layoutRecipientAddressColumns({
      fields: [
        {
          id: 'recipient.address1',
          text,
          shown: true,
          anchorMm: { x: anchorX, y: 0 },
        },
      ],
      specPitchMm: 8,
      bottomYMm: 1,
      leftLimitMm,
      nameLeftMm: 30,
      initialFontSizePt: font,
    })

    expect(result.fontSizePt).toBe(MIN_RECIPIENT_ADDRESS_FONT_PT)
    const minPitch =
      verticalCharAdvanceMm(MIN_RECIPIENT_ADDRESS_FONT_PT) + ADDRESS_COLUMN_MIN_GAP_MM
    expect(result.columnPitchMm).toBeCloseTo(minPitch)
    expect(placedAddressText(result, 'recipient.address1')).toBe(text)
  })

  it('stops at the minimum pitch instead of overlapping glyphs', () => {
    const result = layoutRecipientAddressColumns({
      fields: [
        {
          id: 'recipient.address1',
          text: 'あ'.repeat(30),
          shown: true,
          anchorMm: { x: 40, y: 0 },
        },
      ],
      specPitchMm: 8,
      bottomYMm: 1,
      leftLimitMm: 5,
      nameLeftMm: 12,
      initialFontSizePt: 6,
    })

    const minPitch =
      verticalCharAdvanceMm(MIN_RECIPIENT_ADDRESS_FONT_PT) + ADDRESS_COLUMN_MIN_GAP_MM
    expect(result.columnPitchMm).toBeCloseTo(minPitch)
    expect(12 - result.nameShiftLeftMm).toBeLessThan(5)
    expect(placedAddressText(result, 'recipient.address1')).toHaveLength(30)
  })
})
