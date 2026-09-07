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
        root.unmount()
        throw new Error('プレビューキャンバスの取得に失敗しました')
      }

      const canvas = await html2canvas(canvasEl, {
        scale: CAPTURE_SCALE,
        backgroundColor: '#ffffff',
        useCORS: true,
        logging: false,
      })

      const imgData = canvas.toDataURL('image/png')
      if (i > 0) {
        pdf.addPage([width, height], 'portrait')
      }
      pdf.addImage(imgData, 'PNG', 0, 0, width, height)

      canvas.width = 0
      canvas.height = 0
      root.unmount()
      host.replaceChildren()
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
