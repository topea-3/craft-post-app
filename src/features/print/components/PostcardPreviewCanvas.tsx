import type { CSSProperties, PointerEvent as ReactPointerEvent, ReactNode } from 'react'
import { useCallback, useMemo, useRef } from 'react'
import '@fontsource/noto-serif-jp/japanese-400.css'
import officialPostcardSample from '../../../assets/print/official-postcard-sample.png'
import type { PostcardLayoutSpec } from '../layout/layoutTypes'
import { MM_TO_PT, resultPositionMm, toMm } from '../layout/layoutMath'
import type { PrintLayerId } from '../layout/printLayers'
import { isPostalLayer } from '../layout/printLayers'
import {
  appendCoRecipientJoinMarks,
  isSenderContentLayer,
  isVerticalDashChar,
  NAME_PART_GAP_MM,
  postalDigitOffsetXMm,
  RECIPIENT_POSTAL_LAYOUT,
  scaledFontSizePt,
  scaledSenderContentFontSizePt,
  SENDER_POSTAL_LAYOUT,
  splitPostalDigits,
  verticalTextAdvanceMm,
} from '../layout/printTypography'
import type { LayoutOffsets, LayerVisibility, PrintJobItem } from '../types'

type LayerContent = {
  id: PrintLayerId
  text: string
  hiddenByRule: boolean
}

function buildLayerContents(item: PrintJobItem): LayerContent[] {
  const { address, sender } = item
  let contents: LayerContent[] = [
    { id: 'recipient.postalCode', text: address.postalCode, hiddenByRule: false },
    { id: 'recipient.address1', text: address.addressLine1, hiddenByRule: false },
    { id: 'recipient.address2', text: address.addressLine2, hiddenByRule: !address.addressLine2 },
    { id: 'recipient.address3', text: address.addressLine3, hiddenByRule: !address.addressLine3 },
    { id: 'recipient.primaryLast', text: address.primaryLast, hiddenByRule: false },
    { id: 'recipient.primaryFirst', text: address.primaryFirst, hiddenByRule: false },
    {
      id: 'recipient.honorific',
      text: address.honorificPrint,
      hiddenByRule: !address.honorificPrint,
    },
  ]

  const recipientCoPersonParts: string[][] = []
  for (let n = 1; n <= 3; n++) {
    const co = address.coRecipients[n - 1]
    const lastId = `recipient.coLast.${n}`
    const firstId = `recipient.coFirst.${n}`
    const honorificId = `recipient.coHonorific.${n}`
    recipientCoPersonParts.push([lastId, firstId, honorificId])
    contents.push({
      id: lastId as PrintLayerId,
      text: co?.last ?? '',
      hiddenByRule: !co || co.omitLast,
    })
    contents.push({
      id: firstId as PrintLayerId,
      text: co?.first ?? '',
      hiddenByRule: !co,
    })
    contents.push({
      id: honorificId as PrintLayerId,
      text: address.honorificPrint,
      hiddenByRule: !co || !address.honorificPrint,
    })
  }

  contents.push(
    { id: 'sender.postalCode', text: sender.postalCode, hiddenByRule: false },
    { id: 'sender.address1', text: sender.addressLine1, hiddenByRule: false },
    { id: 'sender.address2', text: sender.addressLine2, hiddenByRule: !sender.addressLine2 },
    { id: 'sender.address3', text: sender.addressLine3, hiddenByRule: !sender.addressLine3 },
    { id: 'sender.primaryLast', text: sender.primaryLast, hiddenByRule: false },
    { id: 'sender.primaryFirst', text: sender.primaryFirst, hiddenByRule: false },
  )

  const senderCoPersonParts: string[][] = []
  for (let n = 1; n <= 4; n++) {
    const co = sender.coRecipients[n - 1]
    const lastId = `sender.coLast.${n}`
    const firstId = `sender.coFirst.${n}`
    senderCoPersonParts.push([lastId, firstId])
    contents.push({
      id: lastId as PrintLayerId,
      text: co?.last ?? '',
      hiddenByRule: !co || co.omitLast,
    })
    contents.push({
      id: firstId as PrintLayerId,
      text: co?.first ?? '',
      hiddenByRule: !co,
    })
  }

  contents = appendCoRecipientJoinMarks(contents, recipientCoPersonParts)
  contents = appendCoRecipientJoinMarks(contents, senderCoPersonParts)
  return contents
}

function isLayerShown(
  layer: LayerContent | undefined,
  visibility: LayerVisibility,
): layer is LayerContent {
  return Boolean(layer && !layer.hiddenByRule && layer.text && visibility[layer.id] !== false)
}

type StackPart = { id: PrintLayerId; text: string; fontSizePt: number; dx: number; dy: number }

function computeColumnPositions(
  parts: StackPart[],
  columnOriginMm: { x: number; y: number },
): Map<PrintLayerId, { x: number; y: number }> {
  const positions = new Map<PrintLayerId, { x: number; y: number }>()
  let y = columnOriginMm.y
  let isFirst = true
  for (const part of parts) {
    if (!isFirst) {
      y += NAME_PART_GAP_MM
    }
    isFirst = false
    positions.set(part.id, {
      x: columnOriginMm.x + toMm(part.dx),
      y: y + toMm(part.dy),
    })
    y += verticalTextAdvanceMm(part.text, part.fontSizePt) + toMm(part.dy)
  }
  return positions
}

function PostalDigits({
  text,
  slotMm,
  groupExtraMm,
}: {
  text: string
  slotMm: number
  groupExtraMm: number
}) {
  const { upper, lower } = splitPostalDigits(text)
  const digits = [...upper, ...lower]
  const widthMm =
    postalDigitOffsetXMm(Math.max(digits.length - 1, 0), slotMm, groupExtraMm) + slotMm
  return (
    <span className="print-postal-digits" style={{ width: `${widthMm}mm`, height: '1em' }}>
      {digits.map((digit, index) => (
        <span
          key={`${index}-${digit}`}
          className="print-postal-digit"
          style={{
            left: `${postalDigitOffsetXMm(index, slotMm, groupExtraMm)}mm`,
            width: `${slotMm}mm`,
          }}
        >
          {digit}
        </span>
      ))}
    </span>
  )
}

function VerticalText({ text }: { text: string }) {
  return (
    <span className="print-vertical-glyphs">
      {[...text].map((ch, index) =>
        isVerticalDashChar(ch) ? (
          <span key={`${index}-${ch}`} className="print-vertical-glyph print-vertical-glyph-dash">
            <span className="print-vertical-glyph-dash-inner">{ch}</span>
          </span>
        ) : (
          <span key={`${index}-${ch}`} className="print-vertical-glyph">
            {ch}
          </span>
        ),
      )}
    </span>
  )
}

type Props = {
  item: PrintJobItem
  layoutSpec: PostcardLayoutSpec
  layoutOffsets: LayoutOffsets
  selectedLayerId: PrintLayerId | null
  onSelectLayer: (id: PrintLayerId) => void
  onOffsetChange: (layerId: PrintLayerId, dx: number, dy: number) => void
  /** When true, used for offscreen PDF capture (no interactive chrome / no sample bg). */
  captureMode?: boolean
  className?: string
}

export function PostcardPreviewCanvas({
  item,
  layoutSpec,
  layoutOffsets,
  selectedLayerId,
  onSelectLayer,
  onOffsetChange,
  captureMode = false,
  className,
}: Props) {
  const canvasRef = useRef<HTMLDivElement>(null)
  const dragRef = useRef<{
    layerId: PrintLayerId
    startX: number
    startY: number
    originDx: number
    originDy: number
    mmPerPx: number
  } | null>(null)

  const layers = useMemo(() => buildLayerContents(item), [item])
  const layerById = useMemo(() => {
    const map = new Map<PrintLayerId, LayerContent>()
    for (const layer of layers) map.set(layer.id, layer)
    return map
  }, [layers])

  const stackedPositions = useMemo(() => {
    const result = new Map<PrintLayerId, { x: number; y: number }>()

    const buildParts = (ids: PrintLayerId[]): StackPart[] =>
      ids
        .map((id) => {
          const layer = layerById.get(id)
          if (!isLayerShown(layer, item.layerVisibility)) return null
          const offset = layoutOffsets[id] ?? { dx: 0, dy: 0 }
          return {
            id,
            text: layer.text,
            fontSizePt: isSenderContentLayer(id)
              ? scaledSenderContentFontSizePt(layoutSpec.layers[id].fontSizePt)
              : scaledFontSizePt(layoutSpec.layers[id].fontSizePt),
            // Relative nudge vs column anchor (anchor keeps its full offset in origin)
            dx: 0,
            dy: 0,
            _absDx: offset.dx,
            _absDy: offset.dy,
          } as StackPart & { _absDx: number; _absDy: number }
        })
        .filter((p): p is StackPart & { _absDx: number; _absDy: number } => p !== null)
        .map((p, index, arr) => {
          const anchor = arr[0]
          return {
            id: p.id,
            text: p.text,
            fontSizePt: p.fontSizePt,
            dx: index === 0 ? 0 : p._absDx - anchor._absDx,
            dy: index === 0 ? 0 : p._absDy - anchor._absDy,
          }
        })

    const primaryIds: PrintLayerId[] = [
      'recipient.primaryLast',
      'recipient.primaryFirst',
      'recipient.honorific',
    ]
    const primaryParts = buildParts(primaryIds)
    let recipientGivenNameStartY: number | null = null
    if (primaryParts.length > 0) {
      const anchorId = primaryParts[0].id
      const anchorPos = resultPositionMm(
        layoutSpec.layers[anchorId].originMm,
        layoutOffsets[anchorId] ?? { dx: 0, dy: 0 },
      )
      for (const [id, pos] of computeColumnPositions(primaryParts, anchorPos)) {
        result.set(id, pos)
      }
      // 連名開始は親の「名」の先頭に合わせる
      recipientGivenNameStartY =
        result.get('recipient.primaryFirst')?.y ?? result.get(anchorId)?.y ?? null
    }

    // 宛名連名は1列（改行なし・・連結）。開始 Y は宛名の「名」に合わせる
    const recipientCoIds: PrintLayerId[] = []
    for (let n = 1; n <= 3; n++) {
      recipientCoIds.push(
        `recipient.coLast.${n}` as PrintLayerId,
        `recipient.coFirst.${n}` as PrintLayerId,
        `recipient.coHonorific.${n}` as PrintLayerId,
      )
    }
    const recipientCoParts = buildParts(recipientCoIds)
    if (recipientCoParts.length > 0) {
      const anchorId = recipientCoParts[0].id
      const anchorPos = resultPositionMm(
        layoutSpec.layers[anchorId].originMm,
        layoutOffsets[anchorId] ?? { dx: 0, dy: 0 },
      )
      const columnOrigin = {
        x: anchorPos.x,
        y: recipientGivenNameStartY ?? anchorPos.y,
      }
      for (const [id, pos] of computeColumnPositions(recipientCoParts, columnOrigin)) {
        result.set(id, pos)
      }
    }

    // 差出人姓名を1列に
    const senderNameIds: PrintLayerId[] = ['sender.primaryLast', 'sender.primaryFirst']
    const senderNameParts = buildParts(senderNameIds)
    let senderGivenNameStartY: number | null = null
    if (senderNameParts.length > 0) {
      const anchorId = senderNameParts[0].id
      const anchorPos = resultPositionMm(
        layoutSpec.layers[anchorId].originMm,
        layoutOffsets[anchorId] ?? { dx: 0, dy: 0 },
      )
      for (const [id, pos] of computeColumnPositions(senderNameParts, anchorPos)) {
        result.set(id, pos)
      }
      senderGivenNameStartY =
        result.get('sender.primaryFirst')?.y ?? result.get(anchorId)?.y ?? null
    }

    // 差出人連名は姓名から改行（別列）し、連名同士は1列で・連結。開始 Y は「名」に合わせる
    const senderCoIds: PrintLayerId[] = []
    for (let n = 1; n <= 4; n++) {
      senderCoIds.push(
        `sender.coLast.${n}` as PrintLayerId,
        `sender.coFirst.${n}` as PrintLayerId,
      )
    }
    const senderCoParts = buildParts(senderCoIds)
    if (senderCoParts.length > 0) {
      const anchorId = senderCoParts[0].id
      const anchorPos = resultPositionMm(
        layoutSpec.layers[anchorId].originMm,
        layoutOffsets[anchorId] ?? { dx: 0, dy: 0 },
      )
      const columnOrigin = {
        x: anchorPos.x,
        y: senderGivenNameStartY ?? anchorPos.y,
      }
      for (const [id, pos] of computeColumnPositions(senderCoParts, columnOrigin)) {
        result.set(id, pos)
      }
    }

    return result
  }, [item.layerVisibility, layerById, layoutOffsets, layoutSpec.layers])

  const handlePointerDown = useCallback(
    (e: ReactPointerEvent<HTMLDivElement>, layerId: PrintLayerId) => {
      if (captureMode) return
      e.preventDefault()
      e.stopPropagation()
      onSelectLayer(layerId)
      const el = canvasRef.current
      if (!el) return
      const rect = el.getBoundingClientRect()
      const mmPerPx = layoutSpec.postcard.width / rect.width
      const offset = layoutOffsets[layerId]
      dragRef.current = {
        layerId,
        startX: e.clientX,
        startY: e.clientY,
        originDx: offset.dx,
        originDy: offset.dy,
        mmPerPx,
      }
      e.currentTarget.setPointerCapture(e.pointerId)
    },
    [captureMode, layoutOffsets, layoutSpec.postcard.width, onSelectLayer],
  )

  const handlePointerMove = useCallback(
    (e: ReactPointerEvent<HTMLDivElement>) => {
      const drag = dragRef.current
      if (!drag) return
      const deltaPxX = e.clientX - drag.startX
      const deltaPxY = e.clientY - drag.startY
      const deltaMmX = deltaPxX * drag.mmPerPx
      const deltaMmY = deltaPxY * drag.mmPerPx
      onOffsetChange(
        drag.layerId,
        drag.originDx + deltaMmX * MM_TO_PT,
        drag.originDy + deltaMmY * MM_TO_PT,
      )
    },
    [onOffsetChange],
  )

  const handlePointerUp = useCallback((e: ReactPointerEvent<HTMLDivElement>) => {
    if (dragRef.current) {
      dragRef.current = null
      try {
        e.currentTarget.releasePointerCapture(e.pointerId)
      } catch {
        /* ignore */
      }
    }
  }, [])

  const renderLayerBody = (layer: LayerContent): ReactNode => {
    if (layer.id === 'recipient.postalCode') {
      return (
        <PostalDigits
          text={layer.text}
          slotMm={RECIPIENT_POSTAL_LAYOUT.slotMm}
          groupExtraMm={RECIPIENT_POSTAL_LAYOUT.groupExtraMm}
        />
      )
    }
    if (layer.id === 'sender.postalCode') {
      return (
        <PostalDigits
          text={layer.text}
          slotMm={SENDER_POSTAL_LAYOUT.slotMm}
          groupExtraMm={SENDER_POSTAL_LAYOUT.groupExtraMm}
        />
      )
    }
    if (isPostalLayer(layer.id)) {
      return layer.text
    }
    return <VerticalText text={layer.text} />
  }

  return (
    <div
      className={`print-preview-canvas-wrap${className ? ` ${className}` : ''}${
        captureMode ? ' print-preview-canvas-wrap-capture' : ''
      }`}
    >
      <div
        ref={canvasRef}
        className="print-preview-canvas"
        style={
          {
            '--postcard-w': `${layoutSpec.postcard.width}mm`,
            '--postcard-h': `${layoutSpec.postcard.height}mm`,
          } as CSSProperties
        }
        data-print-canvas="true"
      >
        {!captureMode && (
          <img
            className="print-preview-postcard-guide"
            src={officialPostcardSample}
            alt=""
            aria-hidden="true"
            draggable={false}
          />
        )}
        {layers.map((layer) => {
          if (!isLayerShown(layer, item.layerVisibility)) return null

          const spec = layoutSpec.layers[layer.id]
          const offset = layoutOffsets[layer.id] ?? { dx: 0, dy: 0 }
          const postalLayout =
            layer.id === 'recipient.postalCode'
              ? RECIPIENT_POSTAL_LAYOUT
              : layer.id === 'sender.postalCode'
                ? SENDER_POSTAL_LAYOUT
                : null
          const defaultPos = postalLayout
            ? resultPositionMm(postalLayout.originMm, offset)
            : resultPositionMm(spec.originMm, offset)
          const stacked = stackedPositions.get(layer.id)
          const leftMm = stacked?.x ?? defaultPos.x
          const topMm = stacked?.y ?? defaultPos.y
          const fontSizePt = postalLayout
            ? postalLayout.fontSizePt
            : isSenderContentLayer(layer.id)
              ? scaledSenderContentFontSizePt(spec.fontSizePt)
              : scaledFontSizePt(spec.fontSizePt)
          const postal = isPostalLayer(layer.id)
          const selected = !captureMode && selectedLayerId === layer.id

          return (
            <div
              key={layer.id}
              className={`print-preview-layer${selected ? ' print-preview-layer-selected' : ''}${
                postal ? ' print-preview-layer-postal' : ' print-preview-layer-vertical'
              }`}
              style={{
                left: `${leftMm}mm`,
                top: `${topMm}mm`,
                fontSize: `${fontSizePt}pt`,
              }}
              onPointerDown={(e) => handlePointerDown(e, layer.id)}
              onPointerMove={handlePointerMove}
              onPointerUp={handlePointerUp}
              onPointerCancel={handlePointerUp}
              role={captureMode ? undefined : 'button'}
              tabIndex={captureMode ? undefined : 0}
            >
              {renderLayerBody(layer)}
            </div>
          )
        })}
      </div>
    </div>
  )
}
