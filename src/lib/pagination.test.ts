/**
 * pagination.ts の clamp 契約を Node の組み込み test で検証する。
 * 実行: node --experimental-strip-types --test src/lib/pagination.test.ts
 */
import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { clampPage, totalPagesFor } from './pagination.ts'

describe('totalPagesFor', () => {
  it('returns at least 1 even when total is 0', () => {
    assert.equal(totalPagesFor(0, 20), 1)
  })

  it('rounds up partial pages', () => {
    assert.equal(totalPagesFor(21, 20), 2)
    assert.equal(totalPagesFor(40, 20), 2)
  })
})

describe('clampPage', () => {
  it('returns 1 when there are no items', () => {
    assert.equal(clampPage(5, 0, 20), 1)
  })

  it('keeps page when it is within range', () => {
    assert.equal(clampPage(2, 40, 20), 2)
  })

  it('clamps to last page after deleting the only item on the last page', () => {
    // 21件 → page2。最後の1件削除で 20件 → page2 は無効、page1 へ
    assert.equal(clampPage(2, 20, 20), 1)
  })

  it('clamps page below 1 up to 1', () => {
    assert.equal(clampPage(0, 10, 20), 1)
  })
})
