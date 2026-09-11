# ライセンス方針（TOP-33）

Craft Post App 本体の配布ライセンスを定義した記録です。

---

## 決定

| 項目 | 内容 |
|------|------|
| **本体ライセンス** | **MIT** |
| **根拠** | 直接依存・主要推移依存の大半が MIT / Apache-2.0 / ISC / BSD 系で、MIT 選択と整合する。GPL 系のブロッカーは検出していない。 |
| **リポジトリ上の表記** | ルート `LICENSE`、`src-tauri/Cargo.toml` の `license = "MIT"` |

## 依存関係の確認結果（要約）

| レイヤ | 結果 |
|--------|------|
| npm（直接依存） | React / Router / html2canvas / jspdf 等は MIT。`@tauri-apps/api` は Apache-2.0 OR MIT。 |
| npm（全体） | MIT が最多。GPL/AGPL/LGPL の fail-on チェックは通過。 |
| 埋め込みフォント | `@fontsource/noto-serif-jp` は **OFL-1.1**。本体 MIT とは別枠。**埋め込み再配布のため著作権表示と OFL 全文の同梱が必須**（下記）。 |
| Rust（主要） | Tauri / serde / sqlx 等は MIT または Apache-2.0 OR MIT が一般的。 |

## 埋め込みフォント（OFL-1.1）の遵守

OFL 条件 2 は、Font Software をソフトウェアに同梱・再配布する場合、**各コピーに著作権表示と本ライセンスを含める**ことを求める（スタンドアロンテキスト可）。

| 成果物 | 役割 |
|--------|------|
| `third_party/noto-serif-jp/OFL.txt` | npm パッケージ付属の著作権表示 + OFL 1.1 全文（正本） |
| `third_party/noto-serif-jp/README.md` | 出所・同梱方針 |
| `THIRD_PARTY_NOTICES.md` | 第三者通知の入口 |
| `tauri.conf.json` → `bundle.resources` | インストーラ／配布物への添付 |

参考 URL: https://scripts.sil.org/OFL （全文の正本は `OFL.txt`）

## 注意

- 依存を追加するときは、GPL 系など相互運用しにくいライセンスが入らないか確認する。
- OFL フォントを追加・差し替える場合は、当該パッケージの著作権表示とライセンス全文を `third_party/` に置き、`THIRD_PARTY_NOTICES.md` と `bundle.resources` を更新する。

## 変更履歴

| 日付 | 内容 |
|------|------|
| 2026-09-12 | TOP-33 に基づき MIT を採用・記録。 |
| 2026-09-12 | OFL フォントの著作権表示・ライセンス全文同梱を必須として追記・実装。 |
