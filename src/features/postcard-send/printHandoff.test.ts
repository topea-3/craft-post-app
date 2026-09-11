import { beforeEach, describe, expect, it } from 'vitest'
import {
  clearSendStatusSelectedIds,
  readPrintJobDraftAddressIds,
  readSendStatusSelectedIds,
  replacePrintJobDraftAddressIds,
  writeSendStatusSelectedIds,
} from './printHandoff'

describe('printHandoff', () => {
  beforeEach(() => {
    sessionStorage.clear()
  })

  it('writes excludedAlerts when replacing printJobDraft', () => {
    replacePrintJobDraftAddressIds(['a', 'b', 'a'])
    const raw = sessionStorage.getItem('printJobDraft')
    expect(JSON.parse(raw!)).toEqual({
      addressEntryIds: ['a', 'b'],
      excludedAlerts: [],
    })
    expect(readPrintJobDraftAddressIds()).toEqual(['a', 'b'])
  })

  it('clears send status selected ids', () => {
    writeSendStatusSelectedIds(['x', 'y'])
    expect(readSendStatusSelectedIds()).toEqual(['x', 'y'])
    clearSendStatusSelectedIds()
    expect(readSendStatusSelectedIds()).toEqual([])
  })
})
