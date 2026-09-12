import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { PostcardReceiptDetailPage } from './PostcardReceiptDetailPage'
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
    received_at: '2025-01-03',
    category: 'nenga',
    memo: 'メモ',
    created_at: '2025-01-03T00:00:00Z',
    updated_at: '2025-01-03T00:00:00Z',
    address_entry_display_name: null,
    address_entry_address_line: null,
    address_entry_archived: null,
    ...overrides,
  }
}

describe('PostcardReceiptDetailPage', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    vi.spyOn(window, 'alert').mockImplementation(() => {})
  })

  it('does not navigate or alert after unmount during delete', async () => {
    const user = userEvent.setup()
    let resolveDelete: (() => void) | undefined
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_postcard_receipt') {
        return Promise.resolve(sampleDto())
      }
      if (cmd === 'delete_postcard_receipt') {
        return new Promise<void>((resolve) => {
          resolveDelete = resolve
        })
      }
      return Promise.reject(new Error(`unexpected ${cmd}`))
    })

    const { unmount } = render(
      <MemoryRouter initialEntries={['/receipts/receipt-1']}>
        <Routes>
          <Route path="/receipts/:id" element={<PostcardReceiptDetailPage />} />
          <Route path="/receipts" element={<div>一覧</div>} />
        </Routes>
      </MemoryRouter>,
    )

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '受取履歴詳細' })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: '削除' }))
    expect(screen.getByRole('button', { name: '削除中…' })).toBeDisabled()
    expect(screen.getByRole('button', { name: '編集' })).toBeDisabled()
    expect(screen.getByRole('button', { name: '戻る' })).toBeDisabled()

    // 二重実行されない
    await user.click(screen.getByRole('button', { name: '削除中…' }))
    expect(invokeMock.mock.calls.filter((c) => c[0] === 'delete_postcard_receipt')).toHaveLength(1)

    unmount()
    await act(async () => {
      resolveDelete?.()
    })

    expect(screen.queryByText('一覧')).not.toBeInTheDocument()
    expect(window.alert).not.toHaveBeenCalled()
  })
})
