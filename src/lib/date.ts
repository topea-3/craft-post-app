/**
 * 日付・時刻ヘルパー。
 *
 * 契約:
 * - バックエンドのタイムスタンプは UTC（RFC3339、末尾 Z）で受け渡す
 * - 表示・カレンダー日の判定はユーザー端末のローカルタイムゾーンで行う
 * - 日付のみ（YYYY-MM-DD）はタイムゾーンを持たないカレンダー日として扱う
 */

const DATE_TIME_FORMAT: Intl.DateTimeFormatOptions = {
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
}

/** 端末のローカルタイムゾーン ID（例: Asia/Tokyo） */
export function getUserTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone
}

/**
 * UTC の RFC3339 / ISO 文字列を、ユーザーのローカルタイムゾーンで表示用に整形する。
 */
export function formatUtcToLocalDateTime(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value

  return new Intl.DateTimeFormat('ja-JP', {
    ...DATE_TIME_FORMAT,
    timeZone: getUserTimeZone(),
  }).format(date)
}

/** Date をローカルタイムゾーンの YYYY-MM-DD にする */
export function formatLocalDate(date: Date = new Date()): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

/** ローカル日付の「今日」（YYYY-MM-DD） */
export function todayLocalDateString(): string {
  return formatLocalDate(new Date())
}

/** YYYY-MM-DD がローカルの今日より未来かどうか */
export function isFutureLocalDate(dateOnly: string): boolean {
  return dateOnly > todayLocalDateString()
}

/** 日付のみ（YYYY-MM-DD）を表示用 YYYY/MM/DD にする（TZ 変換なし） */
export function formatDateOnlyDisplay(value: string): string {
  const [y, m, d] = value.split('-')
  if (!y || !m || !d) return value
  return `${y}/${m}/${d}`
}

/** ローカルタイムゾーンの現在年 */
export function currentLocalYear(): number {
  return new Date().getFullYear()
}
