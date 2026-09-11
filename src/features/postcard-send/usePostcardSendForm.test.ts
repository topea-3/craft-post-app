import { StrictMode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { usePostcardSendForm } from './usePostcardSendForm'
import type { PostcardSendFormValues } from './types'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('usePostcardSendForm', () => {
  it('does not enter an infinite update loop on create mode mount', async () => {
    const onSuccess = vi.fn()
    let renders = 0
    const { result } = renderHook(
      () => {
        renders += 1
        return usePostcardSendForm({
          mode: 'create',
          onSuccess,
        })
      },
      { wrapper: StrictMode },
    )

    await waitFor(() => {
      expect(result.current.values.postcardType).toBe('nenga')
    })

    const rendersAfterMount = renders
    expect(rendersAfterMount).toBeLessThan(10)

    await act(async () => {
      result.current.updateMemo('安定確認')
    })
    expect(result.current.values.memo).toBe('安定確認')
    expect(result.current.isDirty).toBe(true)
    expect(renders - rendersAfterMount).toBeLessThan(6)
  })

  it('does not enter an infinite update loop on edit mode mount', async () => {
    const onSuccess = vi.fn()
    const initialValues: PostcardSendFormValues = {
      sentOn: '2026-09-08',
      postcardType: 'nenga',
      memo: '',
      addressEntryId: 'addr-1',
      addressEntryDisplayName: '誰々 何某 様',
      senderEntryId: 'sender-1',
      senderEntryLabel: 'テスト差出人変更',
    }
    let renders = 0
    const { result, rerender } = renderHook(
      (props: { initialValues: PostcardSendFormValues }) => {
        renders += 1
        return usePostcardSendForm({
          mode: 'edit',
          sendId: 'send-1',
          expectedUpdatedAt: '2026-09-08T12:00:00Z',
          initialValues: props.initialValues,
          baselineSentOn: '2026-09-08',
          onSuccess,
        })
      },
      {
        wrapper: StrictMode,
        initialProps: { initialValues },
      },
    )

    await waitFor(() => {
      expect(result.current.values.sentOn).toBe('2026-09-08')
    })
    const afterMount = renders
    expect(afterMount).toBeLessThan(10)

    // 親が同じ内容の新オブジェクトを渡しても同期 effect が回らないこと
    rerender({
      initialValues: { ...initialValues },
    })
    await act(async () => {})
    expect(renders - afterMount).toBeLessThan(6)

    await act(async () => {
      result.current.updateMemo('編集メモ')
    })
    expect(result.current.values.memo).toBe('編集メモ')
  })

  it('does not update address when overwrite confirm is cancelled', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue(null)
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false)

    const { result } = renderHook(() =>
      usePostcardSendForm({
        mode: 'create',
        onSuccess: vi.fn(),
      }),
    )

    await act(async () => {
      result.current.setAddressEntry('addr-1', '最初の宛名')
    })
    await waitFor(() => {
      expect(result.current.values.addressEntryId).toBe('addr-1')
    })

    await act(async () => {
      result.current.setSenderEntry('sender-1', '手動差出人')
    })
    expect(result.current.values.senderEntryId).toBe('sender-1')

    await act(async () => {
      result.current.setAddressEntry('addr-2', '変更先の宛名')
    })

    expect(confirmSpy).toHaveBeenCalled()
    expect(result.current.values.addressEntryId).toBe('addr-1')
    expect(result.current.values.addressEntryDisplayName).toBe('最初の宛名')
    expect(result.current.values.senderEntryId).toBe('sender-1')
    confirmSpy.mockRestore()
  })
})
