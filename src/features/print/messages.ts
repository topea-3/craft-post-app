export const PRINT_OPERATION_ERROR_MESSAGE =
  '印刷処理に失敗しました。時間をおいて再度お試しください。'

export const PRINT_SELECT_EMPTY_MESSAGE = '1 件以上選択してください。'
export const PRINT_SELECT_MAX_MESSAGE = '最大 200 件まで選択できます。'
export const PRINT_SELECT_LABELS_PENDING_MESSAGE =
  '差出人の確認中です。しばらくしてから再度お試しください。'
export const PRINT_SELECT_NO_OK_ON_PAGE_MESSAGE =
  'このページに選択可能なOK行がありません。'
export const PRINT_NO_VALID_ITEMS_MESSAGE =
  '印刷可能な宛名がありません。差出人の紐づけを確認してください。'
export const PRINT_PRUNE_MESSAGE = (count: number) =>
  `${count} 件は削除またはアーカイブ済みのため選択から外しました。`
export const PRINT_RESOLVE_INVALID_MESSAGE =
  '選択に無効な宛名が含まれていたため除外しました。残りで再度お試しください。'
export const PRINT_PDF_FAILED_MESSAGE = 'PDF の生成に失敗しました。'
export const PRINT_SEND_FAILED_MESSAGE =
  'PDF は生成済みですが、送付記録に失敗しました。再試行してください。'
export const PRINT_PDF_SAVE_FAILED_AFTER_SEND_MESSAGE =
  '送付記録は完了しました。PDF の保存に失敗したので、再ダウンロードしてください。'
export const PRINT_RESNAPSHOT_FAILED_MESSAGE =
  '印刷対象に無効な宛名が含まれています。印刷を中止しました。'
export const PRINT_UNSAVED_LEAVE_MESSAGE =
  '未保存の位置調整があります。このまま移動しますか？'
export const PRINT_TYPE_CHANGE_UNSAVED_MESSAGE =
  '未保存の位置調整があります。種別を切り替えますか？（調整は失われます）'
export const PRINT_PREFS_SAVED_MESSAGE = 'レイアウト調整を保存しました。'
export const PRINT_COMPLETE_MESSAGE =
  'PDF を保存しました。必要に応じて PDF ビューアから印刷してください。'
export const PRINT_EXCLUDED_BANNER = (count: number) =>
  `${count} 件は差出人未紐づけ／archived のため印刷対象外です。`
