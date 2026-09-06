import type { PostcardLayoutSpec } from './layoutTypes'
import { POSTCARD_MARGIN_MM, POSTCARD_SIZE_MM, PRINTABLE_AREA_MM } from './layoutTypes'
import {
  ADDRESS_LINE2_OFFSET_Y_MM,
  ADDRESS_LINE3_OFFSET_Y_MM,
  SENDER_ADDRESS_COL_GAP_MM,
} from './printTypography'
import type { PrintLayerId } from './printLayers'
import { ALL_PRINT_LAYER_IDS } from './printLayers'

/**
 * 年賀状向け宛名面レイアウト（mm、top-left）。
 * fontSizePt は基準値（描画時に LAYOUT_FONT_SCALE。郵便番号は専用レイアウトを使用）。
 */
const layer = (
  x: number,
  y: number,
  fontSizePt: number,
): { originMm: { x: number; y: number }; fontSizePt: number } => ({
  originMm: { x, y },
  fontSizePt,
})

/** 差出人住所先頭: 用紙半分より 10mm 上（差出人エリアを 1cm 上へ） */
const SENDER_ADDRESS_START_Y_MM = POSTCARD_SIZE_MM.height / 2 - 10

/** 差出人住所1 の X（右へ 4mm: 28→32）。列間隔は SENDER_ADDRESS_COL_GAP_MM。 */
const SENDER_ADDRESS1_X_MM = 32

/** 宛名氏名・連名（左へ 3mm: 63.5→60.5 / 51.5→48.5）。連名開始 Y は描画時に「名」へ合わせる */
const RECIPIENT_NAME_X_MM = 60.5
const RECIPIENT_CO_X_MM = 48.5
const RECIPIENT_NAME_Y_MM = 48

/** 差出人姓名列 / 連名列（姓名は住所先頭より 1.5cm 下。連名開始 Y は描画時に「名」へ合わせる） */
const SENDER_NAME_X_MM = 14
const SENDER_CO_X_MM = 8
const SENDER_NAME_Y_MM = SENDER_ADDRESS_START_Y_MM + 15

const nengaLayers: Record<PrintLayerId, { originMm: { x: number; y: number }; fontSizePt: number }> =
  {
    // 宛名郵便番号原点は RECIPIENT_POSTAL_LAYOUT を優先（ここはフォールバック）
    'recipient.postalCode': layer(44.2, 15.2, 11),
    // 宛名住所・氏名など
    'recipient.address1': layer(87.5, 26, 11),
    'recipient.address2': layer(79.5, 26 + ADDRESS_LINE2_OFFSET_Y_MM, 11),
    'recipient.address3': layer(
      71.5,
      26 + ADDRESS_LINE2_OFFSET_Y_MM + ADDRESS_LINE3_OFFSET_Y_MM,
      11,
    ),
    'recipient.primaryLast': layer(RECIPIENT_NAME_X_MM, RECIPIENT_NAME_Y_MM, 18),
    'recipient.primaryFirst': layer(RECIPIENT_NAME_X_MM, RECIPIENT_NAME_Y_MM, 18),
    'recipient.honorific': layer(RECIPIENT_NAME_X_MM, RECIPIENT_NAME_Y_MM, 16),
    // 連名は1列（・で連結）。開始 Y は描画時に親の「名」へ合わせる
    'recipient.coLast.1': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 16),
    'recipient.coFirst.1': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 16),
    'recipient.coHonorific.1': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 14),
    'recipient.coLast.2': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 16),
    'recipient.coFirst.2': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 16),
    'recipient.coHonorific.2': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 14),
    'recipient.coLast.3': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 16),
    'recipient.coFirst.3': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 16),
    'recipient.coHonorific.3': layer(RECIPIENT_CO_X_MM, RECIPIENT_NAME_Y_MM, 14),
    // 差出人郵便番号は SENDER_POSTAL_LAYOUT を優先
    'sender.postalCode': layer(6.2, 121.5, 8),
    // 差出人住所（右へ 4mm、列間隔 -2mm → 4mm）
    'sender.address1': layer(SENDER_ADDRESS1_X_MM, SENDER_ADDRESS_START_Y_MM, 8),
    'sender.address2': layer(
      SENDER_ADDRESS1_X_MM - SENDER_ADDRESS_COL_GAP_MM,
      SENDER_ADDRESS_START_Y_MM + ADDRESS_LINE2_OFFSET_Y_MM,
      8,
    ),
    'sender.address3': layer(
      SENDER_ADDRESS1_X_MM - SENDER_ADDRESS_COL_GAP_MM * 2,
      SENDER_ADDRESS_START_Y_MM + ADDRESS_LINE2_OFFSET_Y_MM + ADDRESS_LINE3_OFFSET_Y_MM,
      8,
    ),
    // 差出人姓名（1列・住所先頭より 1.5cm 下）と連名（改行＝左隣の別列、・で連結）
    'sender.primaryLast': layer(SENDER_NAME_X_MM, SENDER_NAME_Y_MM, 10),
    'sender.primaryFirst': layer(SENDER_NAME_X_MM, SENDER_NAME_Y_MM, 10),
    'sender.coLast.1': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
    'sender.coFirst.1': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
    'sender.coLast.2': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
    'sender.coFirst.2': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
    'sender.coLast.3': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
    'sender.coFirst.3': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
    'sender.coLast.4': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
    'sender.coFirst.4': layer(SENDER_CO_X_MM, SENDER_NAME_Y_MM, 9),
  }

void ALL_PRINT_LAYER_IDS

export const nengaLayoutSpec: PostcardLayoutSpec = {
  postcard: POSTCARD_SIZE_MM,
  marginMm: POSTCARD_MARGIN_MM,
  printable: PRINTABLE_AREA_MM,
  layers: nengaLayers,
}
