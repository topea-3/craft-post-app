import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { PrintSelectPage } from './PrintSelectPage'
import type { AddressEntryDto } from '../../address/types'
import {
  PRINT_SELECT_LABELS_PENDING_MESSAGE,
  PRINT_SELECT_NO_OK_ON_PAGE_MESSAGE,
} from '../messages'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

function addressDto(id: string, last: string): AddressEntryDto {
  return {
    id,
    primary_name: { last, first: '太郎', kana_last: null, kana_first: null },
    co_recipients: [],
    honorific: '様',
    postal_code: '1000001',
    address: {
      prefecture: '東京都',
      city: '千代田区',
      street: '1-1',
      building: null,
    },
    memo: null,
    archived: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  }
}

describe('PrintSelectPage bulk selection', () => {
  beforeEach(() => {
    sessionStorage.clear()
    invokeMock.mockReset()
  })

  it('selects only OK rows on the current page and clears only that page', async () => {
    const user = userEvent.setup()
    sessionStorage.setItem(
      'printJobDraft',
      JSON.stringify({ addressEntryIds: ['other-page'] }),
    )

    invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
      if (cmd === 'search_address_entries') {
        return {
          items: [addressDto('a1', '山田'), addressDto('a2', '佐藤'), addressDto('a3', '鈴木')],
          total: 3,
        }
      }
      if (cmd === 'get_sender_id_by_address_entry_id') {
        const { addressEntryId } = args as { addressEntryId: string }
        if (addressEntryId === 'a3') return null
        return `sender-${addressEntryId}`
      }
      if (cmd === 'get_sender_entry') {
        const { id } = args as { id: string }
        return { label: id, archived: false }
      }
      if (cmd === 'filter_active_address_entry_ids') {
        return (args as { addressEntryIds: string[] }).addressEntryIds
      }
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <PrintSelectPage />
      </MemoryRouter>,
    )

    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '山田 太郎 を選択' })).toBeInTheDocument()
    })
    await waitFor(() => {
      expect(screen.getByText('（未紐づけ）')).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: 'このページのOKを選択' }))

    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '山田 太郎 を選択' })).toBeChecked()
      expect(screen.getByRole('checkbox', { name: '佐藤 太郎 を選択' })).toBeChecked()
      expect(screen.getByRole('checkbox', { name: '鈴木 太郎 を選択' })).not.toBeChecked()
    })
    expect(screen.getByText(/選択: 3 \/ /)).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'このページを解除' }))
    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '山田 太郎 を選択' })).not.toBeChecked()
      expect(screen.getByRole('checkbox', { name: '佐藤 太郎 を選択' })).not.toBeChecked()
    })
    // other-page の選択は残る
    expect(screen.getByText(/選択: 1 \/ /)).toBeInTheDocument()
  })

  it('disables page OK select while sender labels are unresolved', async () => {
    const user = userEvent.setup()
    let resolveSender: (() => void) | null = null
    const senderGate = new Promise<void>((resolve) => {
      resolveSender = resolve
    })

    invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
      if (cmd === 'search_address_entries') {
        return {
          items: [addressDto('a1', '山田')],
          total: 1,
        }
      }
      if (cmd === 'get_sender_id_by_address_entry_id') {
        await senderGate
        return 'sender-a1'
      }
      if (cmd === 'get_sender_entry') {
        return { label: '自宅', archived: false }
      }
      if (cmd === 'filter_active_address_entry_ids') {
        return (args as { addressEntryIds: string[] }).addressEntryIds
      }
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <PrintSelectPage />
      </MemoryRouter>,
    )

    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '山田 太郎 を選択' })).toBeInTheDocument()
    })

    const selectButton = screen.getByRole('button', { name: 'このページのOKを選択' })
    expect(selectButton).toBeDisabled()

    resolveSender?.()
    await waitFor(() => {
      expect(selectButton).toBeEnabled()
    })
    await user.click(selectButton)
    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '山田 太郎 を選択' })).toBeChecked()
    })
    expect(screen.queryByText(PRINT_SELECT_LABELS_PENDING_MESSAGE)).not.toBeInTheDocument()
    expect(screen.queryByText(PRINT_SELECT_NO_OK_ON_PAGE_MESSAGE)).not.toBeInTheDocument()
  })
})
