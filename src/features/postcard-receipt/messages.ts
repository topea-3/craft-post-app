export const POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE =
  '受取履歴の操作に失敗しました。時間をおいて再度お試しください。'

const KNOWN_ERROR_MESSAGES: Record<string, string> = {
  'address entry is archived': '選択した宛名はアーカイブ済みです。',
  'address entry not found': '選択した宛名が見つかりません。',
  'postcard receipt not found': '受取履歴が見つかりませんでした。画面を再読み込みしてください。',
  '受取日に未来の日付は指定できません。': '受取日に未来の日付は指定できません。',
  '送り主の表示名を入力してください。': '送り主の表示名を入力してください。',
  '他の操作で更新済みです。画面を再読み込みしてから再度保存してください。':
    '他の操作で更新済みです。画面を再読み込みしてから再度保存してください。',
}

/** Tauri の拒否文字列をユーザー向け文言へ変換する（未知の内部詳細は出さない） */
export function mapPostcardReceiptInvokeError(error: unknown): string {
  if (typeof error !== 'string') {
    return POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE
  }
  return KNOWN_ERROR_MESSAGES[error] ?? POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE
}
