import { describe, expect, it } from 'vitest'
import { clampPage, totalPagesFor } from './pagination'

describe('totalPagesFor', () => {
  it('returns at least 1 even when total is 0', () => {
    expect(totalPagesFor(0, 20)).toBe(1)
  })

  it('rounds up partial pages', () => {
    expect(totalPagesFor(21, 20)).toBe(2)
    expect(totalPagesFor(40, 20)).toBe(2)
  })
})

describe('clampPage', () => {
  it('returns 1 when there are no items', () => {
    expect(clampPage(5, 0, 20)).toBe(1)
  })

  it('keeps page when it is within range', () => {
    expect(clampPage(2, 40, 20)).toBe(2)
  })

  it('clamps to last page after deleting the only item on the last page', () => {
    // 21件 → page2。最後の1件削除で 20件 → page2 は無効、page1 へ
    expect(clampPage(2, 20, 20)).toBe(1)
  })

  it('clamps page below 1 up to 1', () => {
    expect(clampPage(0, 10, 20)).toBe(1)
  })
})
