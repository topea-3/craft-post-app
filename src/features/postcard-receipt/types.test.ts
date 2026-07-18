import { describe, expect, it } from 'vitest'
import { buildYearOptions } from './types'

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
