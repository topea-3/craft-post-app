import { POSTCARD_MARGIN_MM, POSTCARD_SIZE_MM, type PostcardLayoutSpec } from './layoutTypes'
import { nengaLayoutSpec } from './nengaLayoutSpec'

/**
 * 喪中はがき向けレイアウト。
 * v1 は年賀状と同座標（必要なら将来わずかにオフセット差分を入れる）。
 */
export const mochuLayoutSpec: PostcardLayoutSpec = {
  ...nengaLayoutSpec,
  layers: { ...nengaLayoutSpec.layers },
  // 喪中はがきに年賀の下部表記はない。余白の手前まで使う。
  recipientAddressBottomMm: POSTCARD_SIZE_MM.height - POSTCARD_MARGIN_MM,
}
