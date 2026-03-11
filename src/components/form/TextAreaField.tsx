type Props = {
  label: string
  value: string
  onChange: (value: string) => void
  name?: string
  rows?: number
  maxLength?: number
  required?: boolean
  error?: string
  helperText?: string
}

export function TextAreaField({
  label,
  value,
  onChange,
  name,
  rows = 4,
  maxLength,
  required,
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
        <textarea
          className={`field-input field-textarea ${error ? 'field-input-error' : ''}`}
          name={name}
          rows={rows}
          maxLength={maxLength}
          value={value}
          onChange={(e) => onChange(e.target.value)}
        />
      </label>
      {error ? <p className="field-error">{error}</p> : null}
      {helperText ? <p className="field-helper">{helperText}</p> : null}
    </div>
  )
}

