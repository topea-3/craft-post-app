/** 総件数から総ページ数を求める（0 件でも最低 1 ページ） */
export function totalPagesFor(total: number, pageSize: number): number {
  if (pageSize <= 0) return 1
  return Math.max(1, Math.ceil(total / pageSize))
}

/**
 * 一覧取得用のページ番号を有効範囲に収める。
 * total > 0 かつ page が総ページを超える場合（最終ページの最後の1件削除など）に使う。
 */
export function clampPage(page: number, total: number, pageSize: number): number {
  if (total <= 0) return 1
  const totalPages = totalPagesFor(total, pageSize)
  return Math.min(Math.max(1, page), totalPages)
}
