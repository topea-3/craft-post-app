import { TextAreaField } from '../../components/form/TextAreaField'

type Props = {
  value: string
  error?: string
  onChange: (value: string) => void
}

export function MemoField({ value, error, onChange }: Props) {
  return (
    <TextAreaField
      label="メモ"
      value={value}
      rows={4}
      maxLength={1000}
      error={error}
      helperText="最大 1000 文字まで入力できます。"
      onChange={onChange}
    />
  )
}

