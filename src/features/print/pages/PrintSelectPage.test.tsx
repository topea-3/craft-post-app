import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { PrintSelectPage } from './PrintSelectPage'
import type { AddressEntryDto } from '../../address/types'

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
        return (args as { ids: string[] }).ids
      }
      throw new Error(`unexpected command ${cmd}`)
    })
  })

  it('selects only OK rows on the current page and can clear all', async () => {
    const user = userEvent.setup()
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

    await user.click(screen.getByRole('button', { name: '全選択（OKのみ）' }))

    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '山田 太郎 を選択' })).toBeChecked()
      expect(screen.getByRole('checkbox', { name: '佐藤 太郎 を選択' })).toBeChecked()
      expect(screen.getByRole('checkbox', { name: '鈴木 太郎 を選択' })).not.toBeChecked()
    })
    expect(screen.getByText(/選択: 2 \/ /)).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: '全解除' }))
    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '山田 太郎 を選択' })).not.toBeChecked()
      expect(screen.getByRole('checkbox', { name: '佐藤 太郎 を選択' })).not.toBeChecked()
    })
    expect(screen.getByText(/選択: 0 \/ /)).toBeInTheDocument()
  })
})
