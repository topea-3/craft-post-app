import type { PrintLayerId } from './printLayers'

/** Top-left origin in millimeters (layoutSpec space). */
export type PointMm = {
  x: number
  y: number
}

export type SizeMm = {
  width: number
  height: number
}

export type RectMm = PointMm & SizeMm

/** Offset in points (prefs / job layoutOffsets). */
export type OffsetPt = {
  dx: number
  dy: number
}

export type LayerLayoutSpec = {
  originMm: PointMm
  /** Font size in pt (min 6). */
  fontSizePt: number
}

export type PostcardLayoutSpec = {
  postcard: SizeMm
  marginMm: number
  printable: RectMm
  layers: Record<PrintLayerId, LayerLayoutSpec>
}

export const POSTCARD_SIZE_MM: SizeMm = { width: 100, height: 148 }
export const POSTCARD_MARGIN_MM = 5
export const PRINTABLE_AREA_MM: RectMm = {
  x: POSTCARD_MARGIN_MM,
  y: POSTCARD_MARGIN_MM,
  width: 94,
  height: 142,
}
