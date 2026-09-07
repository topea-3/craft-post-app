import { describe, expect, it } from 'vitest'
import {
  fromResolvePrintJobItemsDto,
  parseAddressEntriesInvalidError,
  type ResolvePrintJobItemsDto,
} from './types'

const sampleItemDto = {
  address: {
    address_entry_id: 'addr-1',
    postal_code: '1234567',
    address_line1: '東京都千代田区',
    address_line2: '1-1-1',
    address_line3: '',
    primary_last: '佐藤',
    primary_first: '一郎',
    co_recipients: [],
    honorific_print: '様',
  },
  sender: {
    sender_entry_id: 'sender-1',
    postal_code: '1234567',
    address_line1: '東京都渋谷区',
    address_line2: '神南1-1-1',
    address_line3: '',
    primary_last: '山田',
    primary_first: '太郎',
    co_recipients: [],
  },
}

describe('parseAddressEntriesInvalidError', () => {
  it('parses ADDRESS_ENTRIES_INVALID JSON payload', () => {
    const raw = JSON.stringify({
      code: 'ADDRESS_ENTRIES_INVALID',
      entries: [
        { address_entry_id: 'a1', reason: 'archived' },
        { address_entry_id: 'a2', reason: 'not_found' },
      ],
    })
    expect(parseAddressEntriesInvalidError(raw)).toEqual({
      code: 'ADDRESS_ENTRIES_INVALID',
      entries: [
        { address_entry_id: 'a1', reason: 'archived' },
        { address_entry_id: 'a2', reason: 'not_found' },
      ],
    })
  })

  it('returns null for non-JSON or unrelated errors', () => {
    expect(parseAddressEntriesInvalidError('plain error')).toBeNull()
    expect(parseAddressEntriesInvalidError(new Error('oops'))).toBeNull()
    expect(
      parseAddressEntriesInvalidError(JSON.stringify({ code: 'OTHER', entries: [] })),
    ).toBeNull()
  })
})

describe('fromResolvePrintJobItemsDto', () => {
  it('maps excluded alerts from excluded', () => {
    const dto: ResolvePrintJobItemsDto = {
      items: [sampleItemDto],
      excluded: [{ address_entry_id: 'x1', reason: 'no_sender_link' }],
    }
    const result = fromResolvePrintJobItemsDto(dto)
    expect(result.items).toHaveLength(1)
    expect(result.items[0].address.addressEntryId).toBe('addr-1')
    expect(result.excludedAlerts).toEqual([
      { addressEntryId: 'x1', reason: 'no_sender_link' },
    ])
  })

  it('prefers excluded_alerts when both are present', () => {
    const dto: ResolvePrintJobItemsDto = {
      items: [],
      excluded: [{ address_entry_id: 'old', reason: 'archived' }],
      excluded_alerts: [{ address_entry_id: 'new', reason: 'sender_archived' }],
    }
    const result = fromResolvePrintJobItemsDto(dto)
    expect(result.excludedAlerts).toEqual([
      { addressEntryId: 'new', reason: 'sender_archived' },
    ])
  })

  it('maps display_name onto excluded alerts', () => {
    const dto: ResolvePrintJobItemsDto = {
      items: [],
      excluded: [
        {
          address_entry_id: 'x1',
          reason: 'no_sender_link',
          display_name: '山田 花子',
        },
      ],
    }
    const result = fromResolvePrintJobItemsDto(dto)
    expect(result.excludedAlerts).toEqual([
      { addressEntryId: 'x1', reason: 'no_sender_link', displayName: '山田 花子' },
    ])
  })
})
