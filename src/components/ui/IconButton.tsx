import type { ButtonHTMLAttributes, ReactNode } from 'react'

type Props = {
  label: string
  children: ReactNode
  variant?: 'normal' | 'primary'
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children' | 'aria-label' | 'title'>

export function IconButton({
  label,
  children,
  variant = 'normal',
  className = '',
  type = 'button',
  ...rest
}: Props) {
  const classes = ['btn', 'btn-icon', variant === 'primary' ? 'btn-primary' : 'btn-normal', className]
    .filter(Boolean)
    .join(' ')

  return (
    <button type={type} className={classes} aria-label={label} title={label} {...rest}>
      {children}
    </button>
  )
}
