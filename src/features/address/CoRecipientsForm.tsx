import type { PersonNameForm } from './types'
import { TextField } from '../../components/form/TextField'

type RowProps = {
  index: number
  value: PersonNameForm
  errors?: {
    last?: string
    first?: string
    kanaLast?: string
    kanaFirst?: string
  }
  onChange: (patch: Partial<PersonNameForm>) => void
  onRemove: () => void
}

function CoRecipientRow({ index, value, errors, onChange, onRemove }: RowProps) {
  return (
    <div className="co-recipient-row">
      <div className="co-recipient-row-header">
        <span>連名 {index + 1}</span>
        <button type="button" className="secondary" onClick={onRemove}>
          削除
        </button>
      </div>
      <div className="grid grid-two-columns">
        <TextField
          label="姓"
          value={value.last}
          error={errors?.last}
          onChange={(v) => onChange({ last: v })}
        />
        <TextField
          label="名"
          value={value.first}
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
    </div>
  )
}

type Props = {
  values: PersonNameForm[]
  errors?: {
    [index: number]: {
      last?: string
      first?: string
      kanaLast?: string
      kanaFirst?: string
    }
  }
  onChangeRow: (index: number, patch: Partial<PersonNameForm>) => void
  onAdd: () => void
  onRemove: (index: number) => void
}

export function CoRecipientsForm({ values, errors = {}, onChangeRow, onAdd, onRemove }: Props) {
  return (
    <div className="co-recipients">
      {values.map((value, index) => (
        <CoRecipientRow
          key={index}
          index={index}
          value={value}
          errors={errors[index]}
          onChange={(patch) => onChangeRow(index, patch)}
          onRemove={() => onRemove(index)}
        />
      ))}
      {values.length < 3 ? (
        <button type="button" className="link-button" onClick={onAdd}>
          ＋連名を追加
        </button>
      ) : null}
    </div>
  )
}

