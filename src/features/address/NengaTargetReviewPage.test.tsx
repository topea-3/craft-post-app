import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { NengaTargetReviewPage } from './NengaTargetReviewPage'
import {
  NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE,
  NENGA_TARGET_REVIEW_TOO_MANY_MESSAGE,
  NENGA_TARGET_YEAR_RANGE_MESSAGE,
} from './messages'

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

  it('shows updated and skipped counts after disable', async () => {
    const user = userEvent.setup()
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'list_nenga_target_review') return [reviewItem('a1', '山田')]
      if (cmd === 'disable_nenga_send_targets') {
        return { updated_count: 1, skipped_count: 2 }
      }
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )
    await screen.findByRole('checkbox', { name: '山田 太郎 様 を年賀状対象外にする' })
    await user.click(screen.getByRole('button', { name: '年賀状送付対象外にする' }))

    expect(await screen.findByRole('status')).toHaveTextContent(
      '1 件を年賀状送付対象外にしました。2 件はすでに対象外かアーカイブ済みのため変更しませんでした。',
    )
  })

  it('shows the fixed load failure message and disables the action', async () => {
    invokeMock.mockRejectedValue('ADDR_NENGA_TARGET_REVIEW_FAILED')
    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )
    expect(await screen.findByText(NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '年賀状送付対象外にする' })).toBeDisabled()
  })

  it('shows the design message when more than 2000 candidates are returned', async () => {
    invokeMock.mockRejectedValue(NENGA_TARGET_REVIEW_TOO_MANY_MESSAGE)
    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )
    expect(await screen.findByText(NENGA_TARGET_REVIEW_TOO_MANY_MESSAGE)).toBeInTheDocument()
    expect(screen.queryByText(NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE)).not.toBeInTheDocument()
  })

  it('does not stay on the loading message when the year is cleared during submit', async () => {
    const user = userEvent.setup()
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    let resolveDisable: (value: { updated_count: number; skipped_count: number }) => void = () => {}
    const disablePromise = new Promise<{ updated_count: number; skipped_count: number }>(
      (resolve) => {
        resolveDisable = resolve
      },
    )
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'list_nenga_target_review') return [reviewItem('a1', '山田')]
      if (cmd === 'disable_nenga_send_targets') return disablePromise
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )
    await screen.findByRole('checkbox', { name: '山田 太郎 様 を年賀状対象外にする' })
    await user.click(screen.getByRole('button', { name: '年賀状送付対象外にする' }))

    const year = screen.getByRole('spinbutton', { name: '年賀状年' })
    await waitFor(() => {
      expect(year).toBeDisabled()
    })
    fireEvent.change(year, { target: { value: '' } })
    resolveDisable({ updated_count: 1, skipped_count: 0 })

    await waitFor(() => {
      expect(screen.queryByText('読み込み中です…')).not.toBeInTheDocument()
    })
    expect(screen.getByText(NENGA_TARGET_YEAR_RANGE_MESSAGE)).toBeInTheDocument()
  })

  it('returns to the first page after a successful disable', async () => {
    const user = userEvent.setup()
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    let listed = false
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === 'list_nenga_target_review') {
        if (!listed) {
          listed = true
          return Array.from({ length: 45 }, (_, index) => reviewItem(`a${index}`, `行${index + 1}`))
        }
        return Array.from({ length: 25 }, (_, index) => reviewItem(`b${index}`, `残${index + 1}`))
      }
      if (cmd === 'disable_nenga_send_targets') {
        return { updated_count: 45, skipped_count: 0 }
      }
      throw new Error(`unexpected command ${cmd}`)
    })

    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )
    await screen.findByRole('checkbox', { name: '行1 太郎 様 を年賀状対象外にする' })
    await user.click(screen.getByRole('button', { name: '次へ' }))
    await user.click(screen.getByRole('button', { name: '次へ' }))
    expect(screen.getByText('3 / 3')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: '年賀状送付対象外にする' }))
    expect(await screen.findByText('1 / 2')).toBeInTheDocument()
    expect(screen.getByRole('checkbox', { name: '残1 太郎 様 を年賀状対象外にする' })).toBeInTheDocument()
  })

  it('reloads the same year after a load failure', async () => {
    const user = userEvent.setup()
    let calls = 0
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd !== 'list_nenga_target_review') throw new Error(`unexpected command ${cmd}`)
      calls += 1
      if (calls === 1) throw 'ADDR_NENGA_TARGET_REVIEW_FAILED'
      return [reviewItem('a1', '山田')]
    })

    render(
      <MemoryRouter>
        <NengaTargetReviewPage />
      </MemoryRouter>,
    )
    expect(await screen.findByText(NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE)).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: '再読み込み' }))
    expect(
      await screen.findByRole('checkbox', { name: '山田 太郎 様 を年賀状対象外にする' }),
    ).toBeChecked()
    expect(screen.queryByText(NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE)).not.toBeInTheDocument()
  })
})
