import { SelectField } from '../../components/form/SelectField'
import { HONORIFIC_OPTIONS } from './types'

type Props = {
  value: string
  error?: string
  onChange: (value: string) => void
}

export function HonorificSelect({ value, error, onChange }: Props) {
  return (
    <SelectField
      label="敬称"
      value={value}
      options={HONORIFIC_OPTIONS}
      onChange={onChange}
      error={error}
    />
  )
}

