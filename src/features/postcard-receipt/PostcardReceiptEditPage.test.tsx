import { render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { PostcardReceiptEditPage } from './PostcardReceiptEditPage'
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

function renderEditPage(id = 'receipt-1') {
  return render(
    <MemoryRouter initialEntries={[`/receipts/${id}/edit`]}>
      <Routes>
        <Route path="/receipts/:id/edit" element={<PostcardReceiptEditPage />} />
      </Routes>
    </MemoryRouter>,
  )
}

describe('PostcardReceiptEditPage', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('loads detail and renders a stable edit form without update-depth loops', async () => {
    invokeMock.mockResolvedValueOnce(sampleDto())

    renderEditPage()

    expect(screen.getByText('読み込み中です…')).toBeInTheDocument()

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '受取履歴編集' })).toBeInTheDocument()
      expect(screen.getByDisplayValue('2025-01-03')).toBeInTheDocument()
      expect(screen.getByDisplayValue('田中家')).toBeInTheDocument()
    })

    expect(invokeMock).toHaveBeenCalledTimes(1)
  })

  it('converges on load failure without update-depth loops', async () => {
    invokeMock.mockRejectedValueOnce('get failed')

    renderEditPage()

    await waitFor(() => {
      expect(screen.getByText('受取履歴の操作に失敗しました。時間をおいて再度お試しください。')).toBeInTheDocument()
    })

    expect(screen.queryByRole('heading', { name: '受取履歴編集' })).not.toBeInTheDocument()
    expect(invokeMock).toHaveBeenCalledTimes(1)
  })
})
