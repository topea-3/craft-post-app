import type { PersonNameForm } from './types'
import { TextField } from '../../components/form/TextField'

type Props = {
  value: PersonNameForm
  errors?: {
    last?: string
    first?: string
    kanaLast?: string
    kanaFirst?: string
  }
  onChange: (patch: Partial<PersonNameForm>) => void
}

export function PrimaryNameForm({ value, errors, onChange }: Props) {
  return (
    <div className="grid grid-two-columns">
      <TextField
        label="姓"
        value={value.last}
        required
        error={errors?.last}
        onChange={(v) => onChange({ last: v })}
      />
      <TextField
        label="名"
        value={value.first}
        required
        error={errors?.first}
        onChange={(v) => onChange({ first: v })}
      />
      <TextField
        label="カナ（姓）"
        value={value.kanaLast ?? ''}
        error={errors?.kanaLast}
        onChange={(v) => onChange({ kanaLast: v })}
      />
      <TextField
        label="カナ（名）"
        value={value.kanaFirst ?? ''}
        error={errors?.kanaFirst}
        onChange={(v) => onChange({ kanaFirst: v })}
      />
    </div>
  )
}

