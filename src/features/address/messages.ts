/**
 * 住所録操作でサーバーエラーが返ったときに表示する汎用メッセージ。
 * 内部エラー詳細をクライアントに表示しないため、固定文言を使用する。
 */
export const ADDRESS_OPERATION_ERROR_MESSAGE =
  '処理に失敗しました。しばらくして再度お試しください。'

export const NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE =
  '見直し一覧の取得に失敗しました。時間をおいて再度お試しください。'
export const NENGA_TARGET_REVIEW_EMPTY_MESSAGE =
  'この年賀状年で、送付対象を外す候補はありません。'
export const NENGA_TARGET_REVIEW_NONE_UPDATED_MESSAGE =
  '更新できる宛名がありませんでした。'
export const NENGA_TARGET_REVIEW_UPDATED_MESSAGE = (count: number) =>
  `${count} 件を年賀状送付対象外にしました。`
export const NENGA_TARGET_REVIEW_SKIPPED_MESSAGE = (count: number) =>
  `${count} 件はすでに対象外かアーカイブ済みのため変更しませんでした。`
export const NENGA_TARGET_REVIEW_CONFIRM_MESSAGE = (count: number) =>
  `選択した ${count} 件の年賀状送付対象を外します。よろしいですか？`
export const NENGA_TARGET_YEAR_RANGE_MESSAGE =
  '年賀状年は 1900 から 2100 の範囲で指定してください。'
