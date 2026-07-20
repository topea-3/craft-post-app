import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { PostcardReceiptListPage } from './PostcardReceiptListPage'
import type { PostcardReceiptDto } from './types'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

function sampleDto(overrides: Partial<PostcardReceiptDto> = {}): PostcardReceiptDto {
  return {
    id: 'receipt-1',
    address_entry_id: null,
    sender_display_name: '田中家',
    received_at: '2012-05-01',
    category: 'nenga',
    memo: null,
    created_at: '2012-05-01T00:00:00Z',
    updated_at: '2012-05-01T00:00:00Z',
    address_entry_display_name: null,
    address_entry_address_line: null,
    address_entry_archived: null,
    ...overrides,
  }
}

describe('PostcardReceiptListPage', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    vi.spyOn(window, 'confirm').mockReturnValue(true)
  })

  it('clears selected year when the last receipt of that year is deleted', async () => {
    const user = userEvent.setup()
    let availableYears = [2012]
    let deleted = false

    invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
      const params = (args ?? {}) as Record<string, unknown>
      if (cmd === 'list_postcard_receipt_years') {
        return availableYears
      }
      if (cmd === 'search_postcard_receipts') {
        if (!deleted && params.year === 2012) {
          return { items: [sampleDto()], total: 1 }
        }
        if (deleted && (params.year === 2012 || params.year == null)) {
          // year クリア後は全件検索で別年のデータが見える想定
          if (params.year === 2012) {
            return { items: [], total: 0 }
          }
          return {
            items: [
              sampleDto({
                id: 'other',
                sender_display_name: '他件',
                received_at: '2024-01-01',
              }),
            ],
            total: 1,
          }
        }
        return { items: [sampleDto()], total: 1 }
      }
      if (cmd === 'delete_postcard_receipt') {
        deleted = true
        availableYears = []
        return undefined
      }
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <PostcardReceiptListPage />
      </MemoryRouter>,
    )

    await user.click(await screen.findByRole('button', { name: 'フィルタ' }))
    const yearSelect = screen.getByRole('combobox', { name: '受取年' }) as HTMLSelectElement
    await user.selectOptions(yearSelect, '2012')

    await waitFor(() => {
      expect(screen.getByText('田中家')).toBeInTheDocument()
    })
    expect(yearSelect.value).toBe('2012')

    await user.click(screen.getByRole('button', { name: '削除' }))

    await waitFor(() => {
      expect(yearSelect.value).toBe('')
    })
    await waitFor(() => {
      expect(screen.getByText('他件')).toBeInTheDocument()
    })

    const searchCalls = invokeMock.mock.calls.filter((c) => c[0] === 'search_postcard_receipts')
    const lastSearch = searchCalls[searchCalls.length - 1]?.[1] as { year: number | null }
    expect(lastSearch.year).toBeNull()
  })

  it('keeps selected year when list_postcard_receipt_years fails after delete', async () => {
    const user = userEvent.setup()
    let yearsCallCount = 0
    let deleted = false

    invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
      const params = (args ?? {}) as Record<string, unknown>
      if (cmd === 'list_postcard_receipt_years') {
        yearsCallCount += 1
        if (yearsCallCount === 1) {
          return [2012]
        }
        throw new Error('years failed')
      }
      if (cmd === 'search_postcard_receipts') {
        if (!deleted && params.year === 2012) {
          return { items: [sampleDto()], total: 1 }
        }
        if (deleted && params.year === 2012) {
          return { items: [], total: 0 }
        }
        return { items: [], total: 0 }
      }
      if (cmd === 'delete_postcard_receipt') {
        deleted = true
        return undefined
      }
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <PostcardReceiptListPage />
      </MemoryRouter>,
    )

    await user.click(await screen.findByRole('button', { name: 'フィルタ' }))
    const yearSelect = screen.getByRole('combobox', { name: '受取年' }) as HTMLSelectElement
    await user.selectOptions(yearSelect, '2012')

    await waitFor(() => {
      expect(screen.getByText('田中家')).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: '削除' }))

    await waitFor(() => {
      expect(screen.getByText(/受取年の取得に失敗しました/)).toBeInTheDocument()
    })
    expect(yearSelect.value).toBe('2012')

    const searchCalls = invokeMock.mock.calls.filter((c) => c[0] === 'search_postcard_receipts')
    const lastSearch = searchCalls[searchCalls.length - 1]?.[1] as { year: number | null }
    expect(lastSearch.year).toBe(2012)
  })
})
