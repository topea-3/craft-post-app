import type { PostcardLayoutSpec } from './layoutTypes'
import { nengaLayoutSpec } from './nengaLayoutSpec'

/**
 * 喪中はがき向けレイアウト。
 * v1 は年賀状と同座標（必要なら将来わずかにオフセット差分を入れる）。
 */
export const mochuLayoutSpec: PostcardLayoutSpec = {
  ...nengaLayoutSpec,
  layers: { ...nengaLayoutSpec.layers },
}
