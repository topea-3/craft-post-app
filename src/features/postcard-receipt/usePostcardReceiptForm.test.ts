import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { usePostcardReceiptForm } from './usePostcardReceiptForm'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

describe('usePostcardReceiptForm', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('counts emoji as one character toward the memo length limit', async () => {
    const onSuccess = vi.fn()
    const { result } = renderHook(() => usePostcardReceiptForm(onSuccess))

    const emoji = '🎉'
    // UTF-16 では .length が 2 になるが、Array.from では 1 文字
    expect(emoji.length).toBe(2)
    expect(Array.from(emoji).length).toBe(1)

    const memoNearLimit = `${'あ'.repeat(999)}${emoji}`
    expect(memoNearLimit.length).toBe(1001)
    expect(Array.from(memoNearLimit).length).toBe(1000)

    act(() => {
      result.current.setLinkMode('displayName')
      result.current.updateSenderDisplayName('送り主')
      result.current.updateMemo(memoNearLimit)
    })

    invokeMock.mockResolvedValueOnce('receipt-new')

    let ok = false
    await act(async () => {
      ok = await result.current.submit()
    })

    expect(ok).toBe(true)
    expect(result.current.errors.memo).toBeUndefined()
    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(onSuccess).toHaveBeenCalledWith('receipt-new')
  })

  it('rejects memo that exceeds 1000 unicode characters including emoji', async () => {
    const onSuccess = vi.fn()
    const { result } = renderHook(() => usePostcardReceiptForm(onSuccess))

    const memoOverLimit = `${'あ'.repeat(1000)}🎉`
    expect(Array.from(memoOverLimit).length).toBe(1001)

    act(() => {
      result.current.setLinkMode('displayName')
      result.current.updateSenderDisplayName('送り主')
      result.current.updateMemo(memoOverLimit)
    })

    let ok = true
    await act(async () => {
      ok = await result.current.submit()
    })

    expect(ok).toBe(false)
    expect(result.current.errors.memo).toContain('1000')
    expect(invokeMock).not.toHaveBeenCalled()
    expect(onSuccess).not.toHaveBeenCalled()
  })

  it('does not call onSuccess if unmounted before submit resolves', async () => {
    const onSuccess = vi.fn()
    let resolveInvoke: ((value: string) => void) | undefined

    invokeMock.mockImplementationOnce(
      () =>
        new Promise<string>((resolve) => {
          resolveInvoke = resolve
        }),
    )

    const { result, unmount } = renderHook(() => usePostcardReceiptForm(onSuccess))

    act(() => {
      result.current.setLinkMode('displayName')
      result.current.updateSenderDisplayName('送り主')
    })

    let submitPromise: Promise<boolean>
    act(() => {
      submitPromise = result.current.submit()
    })

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledTimes(1)
    })

    unmount()

    await act(async () => {
      resolveInvoke?.('receipt-late')
      await submitPromise!
    })

    expect(onSuccess).not.toHaveBeenCalled()
  })
})
