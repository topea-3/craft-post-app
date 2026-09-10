export const POSTCARD_SEND_OPERATION_ERROR_MESSAGE =
  '送付履歴の操作に失敗しました。時間をおいて再度お試しください。'

const KNOWN_ERROR_MESSAGES: Record<string, string> = {
  'address entry is archived': '選択した宛名はアーカイブ済みです。',
  'address entry not found': '選択した宛名が見つかりません。',
  'sender entry is archived': '選択した差出人はアーカイブ済みです。',
  'sender entry not found': '選択した差出人が見つかりません。',
  'postcard send not found': '送付履歴が見つかりませんでした。画面を再読み込みしてください。',
  '送付日に未来の日付は指定できません。': '送付日に未来の日付は指定できません。',
  '差出人が紐づいていない宛名があります。': '差出人が紐づいていない宛名があります。',
  '宛名が重複しています。': '宛名が重複しています。',
  'メモは 1000 文字以内で入力してください。': 'メモは 1000 文字以内で入力してください。',
  '他の操作で更新済みです。画面を再読み込みしてから再度保存してください。':
    '他の操作で更新済みです。画面を再読み込みしてから再度保存してください。',
  'items must contain at least 1 entry': '宛名を1件以上選択してください。',
  'items must not exceed 200 entries': '宛名は最大 200 件までです。',
}

/** Tauri の拒否文字列をユーザー向け文言へ変換する（未知の内部詳細は出さない） */
export function mapPostcardSendInvokeError(error: unknown): string {
  if (typeof error !== 'string') {
    return POSTCARD_SEND_OPERATION_ERROR_MESSAGE
  }
  if (KNOWN_ERROR_MESSAGES[error]) {
    return KNOWN_ERROR_MESSAGES[error]
  }
  for (const [key, message] of Object.entries(KNOWN_ERROR_MESSAGES)) {
    if (error.includes(key)) {
      return message
    }
  }
  return POSTCARD_SEND_OPERATION_ERROR_MESSAGE
}
