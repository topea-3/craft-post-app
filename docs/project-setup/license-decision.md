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

## 配布物へのライセンス同梱

MIT は著作権表示と許諾文をすべてのコピーに含めることが条件。OFL は埋め込み再配布時に著作権表示とライセンス本文の同梱が条件（条件 2）。

| 成果物 | 役割 |
|--------|------|
| ルート `LICENSE` | 本体 MIT の正本（リポジトリ） |
| `third_party/noto-serif-jp/OFL.txt` | OFL 1.1 全文の正本（リポジトリ／上流コピー元） |
| `THIRD_PARTY_NOTICES.md` | 第三者通知の入口 |
| `src-tauri/licenses/*` | **インストーラ／アプリ資源に同梱するコピー**（`bundle.licenseFile` + `bundle.resources`） |

`tauri.conf.json`:

- `bundle.licenseFile`: `licenses/LICENSE`（NSIS ライセンス画面）
- `bundle.resources`: MIT / OFL / THIRD_PARTY_NOTICES

参考 URL: https://scripts.sil.org/OFL （全文の正本は `OFL.txt` / `licenses/noto-serif-jp-OFL.txt`）

## 注意

- 依存を追加するときは、GPL 系など相互運用しにくいライセンスが入らないか確認する。
- OFL フォントを追加・差し替える場合は `third_party/` を更新し、`src-tauri/licenses/` と `THIRD_PARTY_NOTICES.md`・`bundle.resources` を同期する。
- ルート `LICENSE` を更新したら `src-tauri/licenses/LICENSE` も同じ内容に更新する。

## 変更履歴

| 日付 | 内容 |
|------|------|
| 2026-09-12 | TOP-33 に基づき MIT を採用・記録。 |
| 2026-09-12 | OFL フォントの著作権表示・ライセンス全文同梱を必須として追記・実装。 |
| 2026-09-12 | 本体 MIT を `bundle.licenseFile` / `src-tauri/licenses/` に同梱。 |
