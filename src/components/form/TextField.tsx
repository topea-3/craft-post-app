type Props = {
  label: string
  value: string
  onChange: (value: string) => void
  type?: string
  name?: string
  placeholder?: string
  required?: boolean
  maxLength?: number
  error?: string
  helperText?: string
}

export function TextField({
  label,
  value,
  onChange,
  type = 'text',
  name,
  placeholder,
  required,
  maxLength,
  error,
  helperText,
}: Props) {
  return (
    <div className="field">
      <label className="field-label">
        <span>
          {label}
          {required ? <span className="field-required">＊</span> : null}
        </span>
        <input
          className={`field-input ${error ? 'field-input-error' : ''}`}
          type={type}
          name={name}
          value={value}
          placeholder={placeholder}
          required={required}
          aria-required={required}
          maxLength={maxLength}
          onChange={(e) => onChange(e.target.value)}
        />
      </label>
      {error ? <p className="field-error">{error}</p> : null}
      {helperText ? <p className="field-helper">{helperText}</p> : null}
    </div>
  )
}

