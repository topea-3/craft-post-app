import type { OffsetPt, PointMm, RectMm } from './layoutTypes'
import { PRINTABLE_AREA_MM } from './layoutTypes'

/** 1mm ≈ 2.834645669 pt */
export const MM_TO_PT = 2.834645669

export function toPt(mm: number): number {
  return mm * MM_TO_PT
}

export function toMm(pt: number): number {
  return pt / MM_TO_PT
}

/**
 * Clamp offset so that (originMm in pt + offsetPt) stays inside printable rect
 * in top-left layout space. Origin clamp only (v1; text bbox not considered).
 */
export function clampOffset(
  originMm: PointMm,
  offsetPt: OffsetPt,
  printable: RectMm = PRINTABLE_AREA_MM,
): OffsetPt {
  const originXPt = toPt(originMm.x)
  const originYPt = toPt(originMm.y)
  const resultX = originXPt + offsetPt.dx
  const resultY = originYPt + offsetPt.dy

  const minX = toPt(printable.x)
  const minY = toPt(printable.y)
  const maxX = toPt(printable.x + printable.width)
  const maxY = toPt(printable.y + printable.height)

  const clampedX = Math.min(Math.max(resultX, minX), maxX)
  const clampedY = Math.min(Math.max(resultY, minY), maxY)

  return {
    dx: clampedX - originXPt,
    dy: clampedY - originYPt,
  }
}

/** Result position in mm (for CSS left/top). */
export function resultPositionMm(originMm: PointMm, offsetPt: OffsetPt): PointMm {
  return {
    x: originMm.x + toMm(offsetPt.dx),
    y: originMm.y + toMm(offsetPt.dy),
  }
}
