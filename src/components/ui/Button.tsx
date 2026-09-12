import type { ButtonHTMLAttributes, ReactNode } from 'react'

type Props = {
  children: ReactNode
  variant?: 'normal' | 'primary'
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'>

export function Button({
  children,
  variant = 'normal',
  className = '',
  type = 'button',
  ...rest
}: Props) {
  const classes = ['btn', 'btn-label', variant === 'primary' ? 'btn-primary' : 'btn-normal', className]
    .filter(Boolean)
    .join(' ')

  return (
    <button type={type} className={classes} {...rest}>
      {children}
    </button>
  )
}
