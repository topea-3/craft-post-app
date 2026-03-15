import type { AddressFormValues } from './types'
import { TextField } from '../../components/form/TextField'
import { SelectField } from '../../components/form/SelectField'
import { PREFECTURE_OPTIONS } from './types'

type Props = {
  value: AddressFormValues
  errors?: {
    postalCode?: string
    prefecture?: string
    city?: string
    street?: string
    building?: string
  }
  onChange: (patch: Partial<AddressFormValues>) => void
}

export function AddressForm({ value, errors, onChange }: Props) {
  return (
    <div className="address-form">
      <TextField
        label="郵便番号"
        value={value.postalCode}
        required
        error={errors?.postalCode}
        helperText="ハイフンなし7桁（例: 1234567）"
        onChange={(v) => onChange({ postalCode: v })}
      />
      <SelectField
        label="都道府県"
        value={value.prefecture}
        options={PREFECTURE_OPTIONS}
        required
        error={errors?.prefecture}
        onChange={(v) => onChange({ prefecture: v })}
      />
      <div className="grid grid-two-columns">
        <TextField
          label="市区町村"
          value={value.city}
          required
          error={errors?.city}
          onChange={(v) => onChange({ city: v })}
        />
        <TextField
          label="町名・番地"
          value={value.street}
          required
          error={errors?.street}
          onChange={(v) => onChange({ street: v })}
        />
      </div>
      <TextField
        label="建物名・部屋番号"
        value={value.building ?? ''}
        error={errors?.building}
        onChange={(v) => onChange({ building: v })}
      />
    </div>
  )
}

