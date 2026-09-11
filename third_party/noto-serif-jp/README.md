# Noto Serif JP（埋め込みフォント）

本アプリは印刷プレビュー等のため `@fontsource/noto-serif-jp` 経由で **Noto Serif JP** をバンドルする。

## ライセンス遵守

SIL Open Font License 1.1 の条件 2 により、ソフトウェアへの同梱・再配布時は **著作権表示と本ライセンス** を各コピーに含める必要がある。

| ファイル | 内容 |
|----------|------|
| [OFL.txt](./OFL.txt) | `@fontsource/noto-serif-jp` 付属の著作権表示 + OFL 1.1 全文（配布物に同梱する正本） |

参考 URL（全文は `OFL.txt` を正とする）:

- https://scripts.sil.org/OFL
- https://github.com/notofonts/noto-cjk/blob/main/Serif/LICENSE

## 出所

| 項目 | 内容 |
|------|------|
| npm パッケージ | `@fontsource/noto-serif-jp`（現在の lock に固定された版） |
| 上流 | [notofonts/noto-cjk](https://github.com/notofonts/noto-cjk)（Serif / Japanese） |
| 著作権・商標（上流の説明） | フォントの著作権は Adobe、名称の商標は Google（上流 `README-third_party.md` の記載） |

## アプリへの同梱

Tauri の `bundle.resources` でインストーラ／配布物に `OFL.txt` とルートの `THIRD_PARTY_NOTICES.md` を含める。
