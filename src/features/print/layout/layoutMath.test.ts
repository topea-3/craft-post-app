import { describe, expect, it } from 'vitest'
import { clampOffset, MM_TO_PT, toMm, toPt } from './layoutMath'
import { PRINTABLE_AREA_MM } from './layoutTypes'

describe('toPt / toMm', () => {
  it('converts mm to pt and back', () => {
    expect(toPt(1)).toBeCloseTo(MM_TO_PT)
    expect(toMm(MM_TO_PT)).toBeCloseTo(1)
    expect(toMm(toPt(10))).toBeCloseTo(10)
  })
})

describe('clampOffset', () => {
  it('keeps origin inside printable area for large offsets', () => {
    const originMm = { x: 5, y: 5 }
    const clamped = clampOffset(originMm, { dx: 10_000, dy: -10_000 }, PRINTABLE_AREA_MM)

    const resultX = toPt(originMm.x) + clamped.dx
    const resultY = toPt(originMm.y) + clamped.dy
    const minX = toPt(PRINTABLE_AREA_MM.x)
    const minY = toPt(PRINTABLE_AREA_MM.y)
    const maxX = toPt(PRINTABLE_AREA_MM.x + PRINTABLE_AREA_MM.width)
    const maxY = toPt(PRINTABLE_AREA_MM.y + PRINTABLE_AREA_MM.height)

    expect(resultX).toBeCloseTo(maxX)
    expect(resultY).toBeCloseTo(minY)
    expect(resultX).toBeGreaterThanOrEqual(minX)
    expect(resultX).toBeLessThanOrEqual(maxX)
    expect(resultY).toBeGreaterThanOrEqual(minY)
    expect(resultY).toBeLessThanOrEqual(maxY)
  })

  it('leaves in-range offsets unchanged', () => {
    const originMm = { x: 20, y: 30 }
    const offset = { dx: 5, dy: -3 }
    expect(clampOffset(originMm, offset)).toEqual(offset)
  })
})
