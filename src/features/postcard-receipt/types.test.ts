import { describe, expect, it } from 'vitest'
import {
  buildYearOptions,
  formatMemoSnippet,
  resolveSenderDisplayName,
} from './types'

describe('buildYearOptions', () => {
  it('includes current year and distinct available years sorted desc', () => {
    expect(buildYearOptions([2012, 2024, 2024], 2026)).toEqual([
      { value: '', label: '全期間' },
      { value: '2026', label: '2026' },
      { value: '2024', label: '2024' },
      { value: '2012', label: '2012' },
    ])
  })

  it('still offers current year when availableYears is empty', () => {
    expect(buildYearOptions([], 2026)).toEqual([
      { value: '', label: '全期間' },
      { value: '2026', label: '2026' },
    ])
  })
})

describe('formatMemoSnippet', () => {
  it('does not split emoji surrogate pairs at the truncation boundary', () => {
    const memo = `${'あ'.repeat(29)}🎉extra`
    const snippet = formatMemoSnippet(memo, 30)
    expect(snippet.text).toBe(`${'あ'.repeat(29)}🎉`)
    expect(snippet.truncated).toBe(true)
    expect(snippet.text).not.toMatch(/\uFFFD/)
  })
})

describe('resolveSenderDisplayName', () => {
  it('falls back to deleted-address label when linked id cannot resolve context', () => {
    expect(
      resolveSenderDisplayName({
        addressEntryId: 'addr-1',
        senderDisplayName: null,
        addressEntryDisplayName: null,
        addressEntryArchived: null,
      }),
    ).toBe('（削除済みの宛名）')
  })

  it('prefers snapshot senderDisplayName when address context is missing', () => {
    expect(
      resolveSenderDisplayName({
        addressEntryId: 'addr-1',
        senderDisplayName: '昔の表示名',
        addressEntryDisplayName: null,
      }),
    ).toBe('昔の表示名')
  })
})
