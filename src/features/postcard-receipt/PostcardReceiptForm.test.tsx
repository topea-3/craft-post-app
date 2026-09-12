import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { PostcardReceiptForm } from './PostcardReceiptForm'
import type { UsePostcardReceiptFormResult } from './usePostcardReceiptForm'
import { POSTCARD_RECEIPT_FIELD_IDS } from './usePostcardReceiptForm'
import type { PostcardReceiptFormValues } from './types'

function createForm(overrides: Partial<UsePostcardReceiptFormResult> = {}): UsePostcardReceiptFormResult {
  const values: PostcardReceiptFormValues = {
    receivedAt: '',
    category: 'nenga',
    memo: '',
    linkMode: 'displayName',
    addressEntryId: null,
    addressEntryDisplayName: null,
    senderDisplayName: '',
  }
  return {
    values,
    errors: {},
    isSubmitting: false,
    isDirty: false,
    setLinkMode: vi.fn(),
    setAddressEntry: vi.fn(),
    clearAddressEntry: vi.fn(),
    updateReceivedAt: vi.fn(),
    updateCategory: vi.fn(),
    updateSenderDisplayName: vi.fn(),
    updateMemo: vi.fn(),
    submit: vi.fn(async () => false),
    ...overrides,
  }
}

describe('PostcardReceiptForm a11y', () => {
  it('associates validation errors with inputs via aria and alerts', async () => {
    const user = userEvent.setup()
    const form = createForm({
      errors: {
        receivedAt: '受取日を入力してください。',
        senderDisplayName: '送り主の表示名を入力してください。',
      },
      submit: vi.fn(async () => false),
    })

    render(<PostcardReceiptForm form={form} onCancel={vi.fn()} />)

    const receivedAt = document.getElementById(POSTCARD_RECEIPT_FIELD_IDS.receivedAt)
    expect(receivedAt).toHaveAttribute('aria-invalid', 'true')
    expect(receivedAt).toHaveAttribute(
      'aria-describedby',
      `${POSTCARD_RECEIPT_FIELD_IDS.receivedAt}-error`,
    )
    expect(screen.getByText('受取日を入力してください。')).toHaveAttribute('role', 'alert')

    const sender = document.getElementById(POSTCARD_RECEIPT_FIELD_IDS.senderDisplayName)
    expect(sender).toHaveAttribute('aria-invalid', 'true')

    await user.click(screen.getByRole('button', { name: '保存' }))
    expect(form.submit).toHaveBeenCalled()
  })

  it('exposes form-level errors as alert', () => {
    const form = createForm({
      errors: { form: '他の操作で更新済みです。画面を再読み込みしてから再度保存してください。' },
    })
    render(<PostcardReceiptForm form={form} onCancel={vi.fn()} />)
    const alert = screen.getByRole('alert')
    expect(alert).toHaveAttribute('id', POSTCARD_RECEIPT_FIELD_IDS.form)
    expect(alert).toHaveTextContent('他の操作で更新済み')
  })
})
