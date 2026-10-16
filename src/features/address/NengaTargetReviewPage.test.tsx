import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { NengaTargetReviewPage } from './NengaTargetReviewPage'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

function reviewItem(id: string, last: string) {
  return {
    id,
    primary_name: { last, first: '太郎', kana_last: null, kana_first: null },
    co_recipients: [],
    honorific: '様',
    postal_code: '1000001',
    address: { prefecture: '東京都', city: '千代田区', street: '1-1', building: null },
  }
}

describe('NengaTargetReviewPage', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('checks every candidate and disables the button when none remain', async () => {
    const user = userEvent.setup()
    invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
      if (cmd === 'list_nenga_target_review') {
        const year = (args as { year: number }).year
        if (year === 2025) return [reviewItem('b1', '次年')]
        return [reviewItem('a1', '山田'), reviewItem('a2', '佐藤')]
      }
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )

    const yamada = await screen.findByRole('checkbox', { name: '山田 太郎 様 を年賀状対象外にする' })
    const sato = screen.getByRole('checkbox', { name: '佐藤 太郎 様 を年賀状対象外にする' })
    expect(yamada).toBeChecked()
    expect(sato).toBeChecked()

    await user.click(yamada)
    await user.click(sato)
    expect(screen.getByRole('button', { name: '年賀状送付対象外にする' })).toBeDisabled()

    const year = screen.getByRole('spinbutton', { name: '年賀状年' })
    await user.clear(year)
    await user.type(year, '2025')
    await waitFor(() => {
      expect(screen.getByRole('checkbox', { name: '次年 太郎 様 を年賀状対象外にする' })).toBeChecked()
    })
    expect(screen.queryByRole('checkbox', { name: '山田 太郎 様 を年賀状対象外にする' })).not.toBeInTheDocument()
  })

  it('shows the empty state', async () => {
    invokeMock.mockResolvedValue([])
    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )
    expect(await screen.findByText('この年賀状年で、送付対象を外す候補はありません。')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '年賀状送付対象外にする' })).toBeDisabled()
  })
})
