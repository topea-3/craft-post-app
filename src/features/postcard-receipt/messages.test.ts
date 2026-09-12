import { describe, expect, it } from 'vitest'
import {
  POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE,
  mapPostcardReceiptInvokeError,
} from './messages'

describe('mapPostcardReceiptInvokeError', () => {
  // 公開 rejection 文字列の正本は Rust command_tests の assert_eq!（postcard セクション）
  it('maps known backend rejection strings to user-facing messages', () => {
    expect(mapPostcardReceiptInvokeError('address entry is archived')).toBe(
      '選択した宛名はアーカイブ済みです。',
    )
    expect(mapPostcardReceiptInvokeError('postcard receipt not found')).toBe(
      '受取履歴が見つかりませんでした。画面を再読み込みしてください。',
    )
    expect(
      mapPostcardReceiptInvokeError(
        '他の操作で更新済みです。画面を再読み込みしてから再度保存してください。',
      ),
    ).toBe('他の操作で更新済みです。画面を再読み込みしてから再度保存してください。')
  })

  it('does not treat Display-formatted validation errors as known', () => {
    expect(
      mapPostcardReceiptInvokeError('validation error: 受取日に未来の日付は指定できません。'),
    ).toBe(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
  })

  it('falls back for unknown internal errors', () => {
    expect(mapPostcardReceiptInvokeError('RECEIPT_UPDATE_FAILED')).toBe(
      POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE,
    )
    expect(mapPostcardReceiptInvokeError(42)).toBe(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
  })
})
