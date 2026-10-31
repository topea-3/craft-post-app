import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { nengaLayoutSpec } from '../layout/nengaLayoutSpec'
import {
  createDefaultLayerVisibility,
  createDefaultLayoutOffsets,
  type PrintJobItem,
} from '../types'
import { PostcardPreviewCanvas } from './PostcardPreviewCanvas'

function item(addressLine1: string, senderLine1 = '差出人住所'): PrintJobItem {
  return {
    address: {
      addressEntryId: 'addr-1',
      postalCode: '1000001',
      addressLine1,
      addressLine2: '神南1-1-1',
      addressLine3: '',
      primaryLast: '山田',
      primaryFirst: '太郎',
      coRecipients: [],
      honorificPrint: '様',
    },
    sender: {
      senderEntryId: 'sender-1',
      postalCode: '1000002',
      addressLine1: senderLine1,
      addressLine2: '1-1',
      addressLine3: '',
      primaryLast: '佐藤',
      primaryFirst: '花子',
      coRecipients: [],
    },
    layerVisibility: createDefaultLayerVisibility(),
  }
}

function renderCanvas(addressLine1: string, senderLine1?: string) {
  return render(
    <PostcardPreviewCanvas
      item={item(addressLine1, senderLine1)}
      layoutSpec={nengaLayoutSpec}
      layoutOffsets={createDefaultLayoutOffsets()}
      selectedLayerId={null}
      onSelectLayer={() => {}}
      onOffsetChange={() => {}}
      captureMode
    />,
  )
}

describe('PostcardPreviewCanvas recipient address wrap', () => {
  it('keeps a short address on one column and leaves the name at the spec origin', () => {
    const { container } = renderCanvas('東京都千代田区')
    const address = container.querySelector('[data-print-layer="recipient.address1"]')
    const name = container.querySelector('[data-print-layer="recipient.primaryLast"]')

    const given = container.querySelector('[data-print-layer="recipient.primaryFirst"]')
    expect(address?.querySelectorAll('[data-address-column]')).toHaveLength(1)
    expect(address).toHaveStyle({ left: '87.5mm' })
    expect(name).toHaveStyle({ left: '60.5mm' })
    expect(parseFloat((name as HTMLElement).style.top)).toBeLessThan(
      parseFloat((given as HTMLElement).style.top),
    )
  })

  it('wraps a long address line and shifts the next line and the name left', () => {
    const text = 'あ'.repeat(21)
    const { container } = renderCanvas(text, text)
    const address1 = container.querySelector('[data-print-layer="recipient.address1"]')
    const address2 = container.querySelector('[data-print-layer="recipient.address2"]')
    const name = container.querySelector('[data-print-layer="recipient.primaryLast"]')
    const columns = address1?.querySelectorAll('[data-address-column]')

    expect(columns).toHaveLength(2)
    expect(address1?.textContent).toBe(text)
    expect(address1).toHaveStyle({ left: '79.5mm' })
    expect(columns?.[0]).toHaveStyle({ left: '8mm' })
    expect(columns?.[1]).toHaveStyle({ left: '0mm' })
    expect(address2).toHaveStyle({ left: '71.5mm' })
    expect(name).toHaveStyle({ left: '52.5mm' })
  })

  it('wraps a long sender address above the postal boxes and shifts the sender name left', () => {
    const text = 'あ'.repeat(21)
    const { container } = renderCanvas('東京都千代田区', text)
    const sender1 = container.querySelector('[data-print-layer="sender.address1"]')
    const sender2 = container.querySelector('[data-print-layer="sender.address2"]')
    const senderName = container.querySelector('[data-print-layer="sender.primaryLast"]')
    const columns = sender1?.querySelectorAll('[data-address-column]')

    expect(columns).toHaveLength(2)
    expect(sender1?.textContent).toBe(text)
    expect(sender1).toHaveStyle({ left: '28mm' })
    expect(columns?.[0]).toHaveStyle({ left: '4mm' })
    expect(columns?.[1]).toHaveStyle({ left: '0mm' })
    expect(sender2).toHaveStyle({ left: '24mm' })
    expect(senderName).toHaveStyle({ left: '10mm' })
  })
})
