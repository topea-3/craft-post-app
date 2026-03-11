type Option = {
  value: string
  label: string
}

type Props = {
  label: string
  value: string
  options: Option[]
  onChange: (value: string) => void
  name?: string
  required?: boolean
  error?: string
  helperText?: string
}

export function SelectField({
  label,
  value,
  options,
  onChange,
  name,
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
        <select
          className={`field-input ${error ? 'field-input-error' : ''}`}
          name={name}
          value={value}
          onChange={(e) => onChange(e.target.value)}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </label>
      {error ? <p className="field-error">{error}</p> : null}
      {helperText ? <p className="field-helper">{helperText}</p> : null}
    </div>
  )
}

