import { render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { PostcardSendEditPage } from './PostcardSendEditPage'
import type { PostcardSendDto } from './types'
import { POSTCARD_SEND_OPERATION_ERROR_MESSAGE } from './messages'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

function sampleDto(overrides: Partial<PostcardSendDto> = {}): PostcardSendDto {
  return {
    id: 'send-1',
    print_job_id: 'job-1',
    address_entry_id: 'addr-1',
    sender_entry_id: 'sender-1',
    postcard_type: 'nenga',
    sent_on: '2026-09-08',
    source: 'manual',
    memo: null,
    created_at: '2026-09-08T00:00:00Z',
    updated_at: '2026-09-08T12:00:00Z',
    address_entry_display_name: '誰々 何某 様',
    address_entry_address_line: '東京都',
    address_entry_archived: false,
    sender_entry_label: 'テスト差出人変更',
    sender_entry_display_name: null,
    sender_entry_archived: false,
    ...overrides,
  }
}

function renderEditPage(id = 'send-1') {
  return render(
    <MemoryRouter initialEntries={[`/sends/${id}/edit`]}>
      <Routes>
        <Route path="/sends/:id/edit" element={<PostcardSendEditPage />} />
      </Routes>
    </MemoryRouter>,
  )
}

describe('PostcardSendEditPage', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('loads detail and renders a stable edit form without update-depth loops', async () => {
    invokeMock.mockResolvedValueOnce(sampleDto())

    renderEditPage()

    expect(screen.getByText('読み込み中…')).toBeInTheDocument()

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '送付履歴編集' })).toBeInTheDocument()
      expect(screen.getByDisplayValue('2026-09-08')).toBeInTheDocument()
      expect(screen.getByText('誰々 何某 様')).toBeInTheDocument()
      expect(screen.getByText('テスト差出人変更')).toBeInTheDocument()
    })

    expect(invokeMock).toHaveBeenCalledTimes(1)
  })

  it('converges on load failure without update-depth loops', async () => {
    invokeMock.mockRejectedValueOnce('get failed')

    renderEditPage()

    await waitFor(() => {
      expect(screen.getByText(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)).toBeInTheDocument()
    })

    expect(invokeMock).toHaveBeenCalledTimes(1)
  })
})
