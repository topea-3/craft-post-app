/**
 * 氏名（カナ）欄用: ひらがな・カタカナ・スペースのみ許可。
 * - ひらがな: \u3040-\u309F
 * - カタカナ: \u30A0-\u30FF（ー・を含む）
 * - 半角スペース・全角スペース
 */
const KANA_AND_SPACE_REGEX = /^[\u3040-\u309F\u30A0-\u30FF\u0020\u3000]*$/

export function isKanaOnly(value: string): boolean {
  return KANA_AND_SPACE_REGEX.test(value)
}

/**
 * カナ以外の文字を除去した文字列を返す（入力フィルタ用）。
 */
export function sanitizeKana(value: string): string {
  return value.replace(/[^\u3040-\u309F\u30A0-\u30FF\u0020\u3000]/g, '')
}
