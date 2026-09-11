import { beforeEach, describe, expect, it } from 'vitest'
import { readPrintJobDraftAddressIds, replacePrintJobDraftAddressIds } from './printHandoff'

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
})
