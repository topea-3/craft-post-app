import { renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import type { PostcardReceiptDto } from './types'
import { usePostcardReceiptList } from './usePostcardReceiptList'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

function sampleDto(overrides: Partial<PostcardReceiptDto> = {}): PostcardReceiptDto {
  return {
    id: 'receipt-1',
    address_entry_id: null,
    sender_display_name: '田中家',
    received_at: '2025-01-03',
    category: 'nenga',
    memo: null,
    created_at: '2025-01-03T00:00:00Z',
    updated_at: '2025-01-03T00:00:00Z',
    address_entry_display_name: null,
    address_entry_address_line: null,
    address_entry_archived: null,
    ...overrides,
  }
}

describe('usePostcardReceiptList', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('refetches with clamped offset after last-page item deletion leaves page past totalPages', async () => {
    const onPageChange = vi.fn()
    const pageSize = 2

    // 最終ページの最後の1件削除後: page=2 のままだと total=2 / items=[] になる
    invokeMock
      .mockResolvedValueOnce({
        items: [],
        total: 2,
      })
      .mockResolvedValueOnce({
        items: [
          sampleDto({ id: 'a', sender_display_name: 'A', received_at: '2025-01-02' }),
          sampleDto({ id: 'b', sender_display_name: 'B', received_at: '2025-01-01' }),
        ],
        total: 2,
      })

    const { result } = renderHook(() =>
      usePostcardReceiptList({
        searchText: '',
        year: '',
        category: '',
        addressEntryId: null,
        page: 2,
        pageSize,
        onPageChange,
      }),
    )

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
      expect(result.current.items).toHaveLength(2)
    })

    expect(invokeMock).toHaveBeenCalledTimes(2)
    expect(invokeMock).toHaveBeenNthCalledWith(
      1,
      'search_postcard_receipts',
      expect.objectContaining({ limit: pageSize, offset: pageSize }),
    )
    expect(invokeMock).toHaveBeenNthCalledWith(
      2,
      'search_postcard_receipts',
      expect.objectContaining({ limit: pageSize, offset: 0 }),
    )
    expect(onPageChange).toHaveBeenCalledWith(1)
    expect(result.current.total).toBe(2)
    expect(result.current.error).toBeNull()
  })

  it('does not refetch when the requested page is already valid', async () => {
    invokeMock.mockResolvedValueOnce({
      items: [sampleDto()],
      total: 1,
    })

    const onPageChange = vi.fn()
    const { result } = renderHook(() =>
      usePostcardReceiptList({
        searchText: '',
        year: '',
        category: '',
        addressEntryId: null,
        page: 1,
        pageSize: 20,
        onPageChange,
      }),
    )

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
      expect(result.current.items).toHaveLength(1)
    })

    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(onPageChange).not.toHaveBeenCalled()
  })
})
