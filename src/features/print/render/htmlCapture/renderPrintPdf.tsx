import { createRoot } from 'react-dom/client'
import { flushSync } from 'react-dom'
import html2canvas from 'html2canvas'
import { jsPDF } from 'jspdf'
import { PostcardPreviewCanvas } from '../../components/PostcardPreviewCanvas'
import type { PostcardLayoutSpec } from '../../layout/layoutTypes'
import type { LayoutOffsets, PrintJobItem } from '../../types'

const CAPTURE_SCALE = 3.65 // ~350dpi relative to 96dpi CSS px

export type RenderPrintPdfParams = {
  items: PrintJobItem[]
  layoutSpec: PostcardLayoutSpec
  layoutOffsets: LayoutOffsets
  fileName?: string
}

/** Sample pixels; throw if the page looks blank (all near-white). */
function assertCanvasHasInk(canvas: HTMLCanvasElement): void {
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    throw new Error('プレビューキャンバスの描画に失敗しました')
  }
  const { width, height } = canvas
  if (width === 0 || height === 0) {
    throw new Error('プレビューキャンバスの描画に失敗しました（空白）')
  }
  const { data } = ctx.getImageData(0, 0, width, height)
  const step = Math.max(4, Math.floor(data.length / 4 / 5000) * 4)
  for (let i = 0; i < data.length; i += step) {
    const r = data[i]
    const g = data[i + 1]
    const b = data[i + 2]
    const a = data[i + 3]
    if (a > 8 && (r < 250 || g < 250 || b < 250)) {
      return
    }
  }
  throw new Error('プレビューキャンバスの描画に失敗しました（空白）')
}

/**
 * Sequential 1-page html2canvas capture → append to same jsPDF → release canvas.
 * Peak image buffer stays at one page.
 */
export async function renderPrintPdf(params: RenderPrintPdfParams): Promise<jsPDF> {
  const { items, layoutSpec, layoutOffsets } = params
  if (items.length === 0) {
    throw new Error('印刷対象がありません')
  }

  const { width, height } = layoutSpec.postcard
  const pdf = new jsPDF({
    orientation: 'portrait',
    unit: 'mm',
    format: [width, height],
  })

  const host = document.createElement('div')
  host.className = 'print-pdf-capture-host'
  host.setAttribute('aria-hidden', 'true')
  document.body.appendChild(host)

  try {
    for (let i = 0; i < items.length; i++) {
      const item = items[i]
      const pageRoot = document.createElement('div')
      host.replaceChildren(pageRoot)
      const root = createRoot(pageRoot)
      let canvas: HTMLCanvasElement | null = null

      try {
        flushSync(() => {
          root.render(
            <PostcardPreviewCanvas
              item={item}
              layoutSpec={layoutSpec}
              layoutOffsets={layoutOffsets}
              selectedLayerId={null}
              onSelectLayer={() => {}}
              onOffsetChange={() => {}}
              captureMode
            />,
          )
        })

        if (document.fonts?.ready) {
          await document.fonts.ready
        }

        const canvasEl = pageRoot.querySelector('[data-print-canvas="true"]') as HTMLElement | null
        if (!canvasEl) {
          throw new Error('プレビューキャンバスの取得に失敗しました')
        }

        canvas = await html2canvas(canvasEl, {
          scale: CAPTURE_SCALE,
          backgroundColor: '#ffffff',
          useCORS: true,
          logging: false,
          onclone: (_doc, el) => {
            let node: HTMLElement | null = el
            while (node) {
              node.style.visibility = 'visible'
              node.style.clipPath = 'none'
              node = node.parentElement
            }
          },
        })

        assertCanvasHasInk(canvas)

        const imgData = canvas.toDataURL('image/png')
        if (i > 0) {
          pdf.addPage([width, height], 'portrait')
        }
        pdf.addImage(imgData, 'PNG', 0, 0, width, height)
      } finally {
        if (canvas) {
          canvas.width = 0
          canvas.height = 0
        }
        root.unmount()
        host.replaceChildren()
      }
    }
  } finally {
    host.remove()
  }

  return pdf
}

export async function renderAndDownloadPrintPdf(params: RenderPrintPdfParams): Promise<void> {
  const pdf = await renderPrintPdf(params)
  const name = params.fileName ?? `postcard-address-${Date.now()}.pdf`
  pdf.save(name)
}
