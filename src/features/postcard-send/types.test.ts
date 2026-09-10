import { describe, expect, it } from 'vitest'
import { mapPostcardSendInvokeError } from './messages'
import {
  buildHistoryYearOptions,
  buildReceiptYearOptions,
  buildStatusYearOptions,
  formatMemoSnippet,
  fromPostcardSendDto,
} from './types'

describe('buildHistoryYearOptions', () => {
  it('includes all-period and current year', () => {
    expect(buildHistoryYearOptions([2024], 2026)).toEqual([
      { value: '', label: '全期間' },
      { value: '2026', label: '2026' },
      { value: '2024', label: '2024' },
    ])
  })
})

describe('buildStatusYearOptions', () => {
  it('includes this year, last year, available years, and selected year without all-period', () => {
    expect(buildStatusYearOptions([2020], 2026, 2019)).toEqual([
      { value: '2026', label: '2026' },
      { value: '2025', label: '2025' },
      { value: '2020', label: '2020' },
      { value: '2019', label: '2019' },
    ])
  })
})

describe('buildReceiptYearOptions', () => {
  it('unions target year, target-1, available, and current receipt year', () => {
    expect(buildReceiptYearOptions(2026, [2022], 2023)).toEqual([
      { value: '2026', label: '2026' },
      { value: '2025', label: '2025' },
      { value: '2023', label: '2023' },
      { value: '2022', label: '2022' },
    ])
  })
})

describe('formatMemoSnippet', () => {
  it('truncates by unicode code points', () => {
    const memo = `${'あ'.repeat(29)}🎉extra`
    const snippet = formatMemoSnippet(memo, 30)
    expect(snippet.text).toBe(`${'あ'.repeat(29)}🎉`)
    expect(snippet.truncated).toBe(true)
  })
})

describe('fromPostcardSendDto', () => {
  it('maps snake_case dto and normalizes type/source', () => {
    const item = fromPostcardSendDto({
      id: '1',
      print_job_id: 'job',
      address_entry_id: 'a',
      sender_entry_id: 's',
      postcard_type: 'mochu',
      sent_on: '2026-01-01',
      source: 'manual',
      memo: 'm',
      created_at: 'c',
      updated_at: 'u',
      address_entry_display_name: '宛名',
      address_entry_address_line: '住所',
      address_entry_archived: false,
      sender_entry_label: '差出人',
      sender_entry_display_name: null,
      sender_entry_archived: false,
    })
    expect(item.postcardType).toBe('mochu')
    expect(item.source).toBe('manual')
    expect(item.addressEntryDisplayName).toBe('宛名')
  })
})

describe('mapPostcardSendInvokeError', () => {
  it('maps known validation messages', () => {
    expect(mapPostcardSendInvokeError('宛名が重複しています。')).toBe('宛名が重複しています。')
    expect(mapPostcardSendInvokeError('postcard send not found')).toContain('送付履歴が見つかりません')
  })
})
