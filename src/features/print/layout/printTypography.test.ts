import { describe, expect, it } from 'vitest'
import {
  appendCoRecipientJoinMarks,
  isVerticalDashChar,
  LAYOUT_FONT_SCALE,
  postalDigitOffsetXMm,
  scaledFontSizePt,
  scaledSenderContentFontSizePt,
  SENDER_CONTENT_FONT_SCALE,
  splitPostalDigits,
  verticalTextAdvanceMm,
} from './printTypography'
import { toMm } from './layoutMath'

describe('splitPostalDigits', () => {
  it('strips hyphen and splits 3+4', () => {
    expect(splitPostalDigits('123-4567')).toEqual({
      upper: ['1', '2', '3'],
      lower: ['4', '5', '6', '7'],
    })
  })

  it('accepts digits only', () => {
    expect(splitPostalDigits('9876543')).toEqual({
      upper: ['9', '8', '7'],
      lower: ['6', '5', '4', '3'],
    })
  })
})

describe('postalDigitOffsetXMm', () => {
  it('places digits on slot grid with extra gap after the 3rd', () => {
    expect(postalDigitOffsetXMm(0, 6, 2)).toBe(0)
    expect(postalDigitOffsetXMm(2, 6, 2)).toBe(12)
    expect(postalDigitOffsetXMm(3, 6, 2)).toBe(20) // 3*6 + 2
    expect(postalDigitOffsetXMm(6, 6, 2)).toBe(38)
  })
})

describe('vertical dash detection', () => {
  it('recognizes ascii, fullwidth and japanese dashes', () => {
    expect(isVerticalDashChar('-')).toBe(true)
    expect(isVerticalDashChar('ー')).toBe(true)
    expect(isVerticalDashChar('－')).toBe(true)
    expect(isVerticalDashChar('−')).toBe(true)
    expect(isVerticalDashChar('1')).toBe(false)
  })
})

describe('scaledFontSizePt', () => {
  it('applies 1.5x scale', () => {
    expect(scaledFontSizePt(10)).toBeCloseTo(10 * LAYOUT_FONT_SCALE)
  })
})

describe('scaledSenderContentFontSizePt', () => {
  it('applies layout scale then 0.8 for sender content', () => {
    expect(scaledSenderContentFontSizePt(10)).toBeCloseTo(
      10 * LAYOUT_FONT_SCALE * SENDER_CONTENT_FONT_SCALE,
    )
  })
})

describe('verticalTextAdvanceMm', () => {
  it('advances by character count times font em in mm', () => {
    expect(verticalTextAdvanceMm('誰々', 18)).toBeCloseTo(2 * toMm(18))
  })
})

describe('appendCoRecipientJoinMarks', () => {
  it('appends ・ to the last visible part of each co except the final one', () => {
    const layers = [
      { id: 'coLast.1', text: '山田', hiddenByRule: true },
      { id: 'coFirst.1', text: '花子', hiddenByRule: false },
      { id: 'coHonorific.1', text: '様', hiddenByRule: false },
      { id: 'coLast.2', text: '', hiddenByRule: true },
      { id: 'coFirst.2', text: '次郎', hiddenByRule: false },
      { id: 'coHonorific.2', text: '様', hiddenByRule: false },
    ]
    const result = appendCoRecipientJoinMarks(layers, [
      ['coLast.1', 'coFirst.1', 'coHonorific.1'],
      ['coLast.2', 'coFirst.2', 'coHonorific.2'],
    ])
    expect(result.find((l) => l.id === 'coHonorific.1')?.text).toBe('様・')
    expect(result.find((l) => l.id === 'coFirst.1')?.text).toBe('花子')
    expect(result.find((l) => l.id === 'coHonorific.2')?.text).toBe('様')
  })

  it('does nothing when fewer than two co-recipients are visible', () => {
    const layers = [
      { id: 'coFirst.1', text: '花子', hiddenByRule: false },
      { id: 'coFirst.2', text: '', hiddenByRule: true },
    ]
    const result = appendCoRecipientJoinMarks(layers, [['coFirst.1'], ['coFirst.2']])
    expect(result.find((l) => l.id === 'coFirst.1')?.text).toBe('花子')
  })
})
