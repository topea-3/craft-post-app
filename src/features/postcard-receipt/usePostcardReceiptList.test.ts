import { useState } from 'react'
import { act, renderHook, waitFor } from '@testing-library/react'
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

  it('does not refetch while search text is still debouncing', async () => {
    vi.useFakeTimers()
    invokeMock.mockResolvedValue({
      items: [sampleDto()],
      total: 1,
    })

    const onPageChange = vi.fn()
    const { result, rerender } = renderHook(
      ({ searchText, page }: { searchText: string; page: number }) =>
        usePostcardReceiptList({
          searchText,
          year: '',
          category: '',
          addressEntryId: null,
          page,
          pageSize: 20,
          onPageChange,
        }),
      { initialProps: { searchText: '', page: 2 } },
    )

    await act(async () => {
      await Promise.resolve()
    })
    expect(result.current.isLoading).toBe(false)
    expect(invokeMock).toHaveBeenCalledTimes(1)
    invokeMock.mockClear()
    onPageChange.mockClear()

    rerender({ searchText: '田中', page: 2 })

    await act(async () => {
      await vi.advanceTimersByTimeAsync(299)
    })
    expect(invokeMock).not.toHaveBeenCalled()
    expect(onPageChange).not.toHaveBeenCalled()

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1)
    })
    expect(onPageChange).toHaveBeenCalledWith(1)

    rerender({ searchText: '田中', page: 1 })
    await act(async () => {
      await Promise.resolve()
    })
    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(invokeMock).toHaveBeenCalledWith(
      'search_postcard_receipts',
      expect.objectContaining({ keyword: '田中', offset: 0 }),
    )

    vi.useRealTimers()
  })

  it('does not show empty-all message while clearing a zero-result search', async () => {
    vi.useFakeTimers()
    invokeMock
      .mockResolvedValueOnce({ items: [], total: 0 })
      .mockResolvedValueOnce({
        items: [sampleDto({ id: 'a', sender_display_name: '実データ' })],
        total: 1,
      })

    const { result, rerender } = renderHook(
      ({ searchText }: { searchText: string }) =>
        usePostcardReceiptList({
          searchText,
          year: '',
          category: '',
          addressEntryId: null,
          page: 1,
          pageSize: 20,
        }),
      { initialProps: { searchText: '存在しない' } },
    )

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300)
      await Promise.resolve()
    })
    expect(result.current.total).toBe(0)
    expect(result.current.settledSearchText).toBe('存在しない')

    rerender({ searchText: '' })
    // debounce 中は settled が旧条件のまま → 空全体メッセージ用の !isFiltering にはならない
    expect(result.current.isDebouncing).toBe(true)
    expect(result.current.settledSearchText).toBe('存在しない')

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300)
      await Promise.resolve()
    })
    expect(result.current.isDebouncing).toBe(false)
    expect(result.current.settledSearchText).toBe('')
    expect(result.current.total).toBe(1)

    vi.useRealTimers()
  })

  it('clears stale items when clamping page after delete on last page', async () => {
    const pageSize = 2
    const staleItem = sampleDto({ id: 'stale', sender_display_name: '削除済み' })

    invokeMock.mockResolvedValueOnce({
      items: [staleItem],
      total: 3,
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
      expect(result.current.list.items).toHaveLength(1)
      expect(result.current.list.items[0]?.id).toBe('stale')
    })

    let resolveClampFetch: (value: { items: PostcardReceiptDto[]; total: number }) => void
    const clampFetchPromise = new Promise<{ items: PostcardReceiptDto[]; total: number }>(
      (resolve) => {
        resolveClampFetch = resolve
      },
    )

    invokeMock.mockReset()
    invokeMock
      .mockImplementationOnce(() => clampFetchPromise)
      .mockResolvedValueOnce({
        items: [
          sampleDto({ id: 'a', sender_display_name: 'A' }),
          sampleDto({ id: 'b', sender_display_name: 'B' }),
        ],
        total: 2,
      })

    act(() => {
      result.current.list.reload()
    })

    await act(async () => {
      resolveClampFetch!({ items: [], total: 2 })
      await Promise.resolve()
    })

    await waitFor(() => {
      expect(result.current.page).toBe(1)
    })
    expect(result.current.list.items.some((item) => item.id === 'stale')).toBe(false)

    await waitFor(() => {
      expect(result.current.list.items).toHaveLength(2)
    })
  })
})
