import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { AddressEntryForm } from './AddressEntryForm'
import { createInitialAddressEntryFormValues } from './types'

describe('AddressEntryForm nenga send target', () => {
  it('starts checked on create', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(
      <AddressEntryForm
        mode="create"
        values={createInitialAddressEntryFormValues()}
        errors={{}}
        isSubmitting={false}
        onSubmit={() => {}}
        onCancel={() => {}}
        onChangePrimaryName={() => {}}
        onChangeAddress={() => {}}
        onAddCoRecipient={() => {}}
        onChangeCoRecipient={() => {}}
        onRemoveCoRecipient={() => {}}
        onChangeHonorific={() => {}}
        onChangeMemo={() => {}}
        onChangeNengaSendTarget={onChange}
      />,
    )

    const checkbox = screen.getByRole('checkbox', { name: '年賀状送付対象' })
    expect(checkbox).toBeChecked()
    await user.click(checkbox)
    expect(onChange).toHaveBeenCalledWith(false)
  })
})
