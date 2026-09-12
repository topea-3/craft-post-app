import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Button } from './Button'
import { IconButton } from './IconButton'
import { IconPlus } from './icons'

describe('Button', () => {
  it('renders a labeled normal button by default', () => {
    render(<Button>保存</Button>)
    const button = screen.getByRole('button', { name: '保存' })
    expect(button).toHaveClass('btn', 'btn-label', 'btn-normal')
  })

  it('applies primary variant for advancing actions', () => {
    render(<Button variant="primary">進む</Button>)
    expect(screen.getByRole('button', { name: '進む' })).toHaveClass('btn-primary')
  })
})

describe('IconButton', () => {
  it('exposes tooltip text via title and aria-label', async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    render(
      <IconButton label="新規作成" onClick={onClick}>
        <IconPlus />
      </IconButton>,
    )

    const button = screen.getByRole('button', { name: '新規作成' })
    expect(button).toHaveAttribute('title', '新規作成')
    expect(button).toHaveClass('btn-icon', 'btn-normal')
    await user.click(button)
    expect(onClick).toHaveBeenCalledOnce()
  })
})
