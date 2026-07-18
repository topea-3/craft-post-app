import { useState } from 'react'
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

  it('clamps page via onPageChange and fetches twice total (not thrice)', async () => {
    const pageSize = 2

    // page=2 で空ページ → onPageChange(1) → effect 再実行で page=1 を取得
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

    const { result } = renderHook(() => {
      const [page, setPage] = useState(2)
      const list = usePostcardReceiptList({
        searchText: '',
        year: '',
        category: '',
        addressEntryId: null,
        page,
        pageSize,
        onPageChange: setPage,
      })
      return { page, list }
    })

    await waitFor(() => {
      expect(result.current.page).toBe(1)
      expect(result.current.list.isLoading).toBe(false)
      expect(result.current.list.items).toHaveLength(2)
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
    expect(result.current.list.total).toBe(2)
    expect(result.current.list.error).toBeNull()
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

  it('notifies onPageChange when clamping without applying empty items', async () => {
    const onPageChange = vi.fn()
    const pageSize = 2

    invokeMock.mockResolvedValueOnce({
      items: [],
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
      expect(onPageChange).toHaveBeenCalledWith(1)
      expect(result.current.isLoading).toBe(false)
    })

    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(result.current.total).toBe(2)
    // 空結果を適用せず、前回の items（初期 []）のまま
    expect(result.current.items).toEqual([])
  })
})
