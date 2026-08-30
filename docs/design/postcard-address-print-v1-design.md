# はがき宛名面印刷（v1）— 設計書

- **Linear Issue**: [TOP-19](https://linear.app/topea-3/issue/TOP-19/v1-はがき宛名面印刷の機能設計) v1 はがき宛名面印刷の機能設計
- **親 Issue**: [TOP-14](https://linear.app/topea-3/issue/TOP-14/v1-機能設計) v1 機能設計
- **後続実装 Issue**: [TOP-28](https://linear.app/topea-3/issue/TOP-28/v1-はがき宛名面印刷機能の実装) v1 はがき宛名面印刷機能の実装
- **関連要求**: Linear Doc [要求仕様](https://linear.app/topea-3/document/要求仕様-0386f26238ae)
- **ステータス**: Draft
- **最終更新**: 2026-08-30

---

## 1. 背景・目的

v1.0.0 では年賀状・喪中はがき等の**宛名面印刷**を、住所録・差出人データと連携して行えるようにする。印刷前にレイアウトを確認・調整でき、DB 経路の印刷完了時には**送付情報**を自動記録する。

本設計は TOP-19 のスコープに限定する。送付情報の一覧 CRUD 全機能・CSV 印刷・デザイン面印刷は別 Issue / 将来とする。

---

## 2. スコープ

### 2.1 対象（v1 / TOP-19）

| 領域 | 内容 |
|------|------|
| レイアウト | 官製はがき宛名面（100×148mm）。年賀状向け縦書き主体＋郵便番号横書き（[要求仕様](https://linear.app/topea-3/document/要求仕様-0386f26238ae)） |
| テンプレート | はがき種別（年賀状 / 喪中はがき）ごとのコード内テンプレート 1 式ずつ |
| 印刷モード | **1 件印刷** / **複数件一括印刷**（同一ジョブ内でページ分割） |
| データ連携 | `AddressEntry`（宛名）・`SenderEntry`（差出人）・印刷時 `PostcardSend` 作成 IF |
| プレビュー | 画面内プレビュー＋レイヤー単位の位置調整・表示 ON/OFF |
| 印刷実行 | PDF 生成（@react-pdf/renderer）→ OS 印刷または PDF 保存 |
| 永続化 | レイアウト位置調整の保存（次回印刷へ反映）。表示 ON/OFF はセッションのみ |

### 2.2 非スコープ

- CSV 読み取り印刷（要求仕様に記載あるが v1.1 の CSV 機能と整合。v1 は DB 経路のみ）
- デザイン面（イラスト・文面）印刷
- ユーザー定義テンプレートの CRUD（コード内テンプレートのみ。TOP-10 方針）
- 送付情報の一覧・編集 UI 全般（送付 Issue で設計。本設計は印刷完了時の**作成 IF** のみ定義）
- はがき受取情報との自動連携
- クラウド・複数端末同期

---

## 3. 要件

### 3.1 機能要件

| ID | 要件 |
|----|------|
| FR-01 | 住所録一覧から 1 件以上の `AddressEntry` を選択し、宛名面印刷フローを開始できる |
| FR-02 | 印刷対象ごと（一括時は各宛名）に差出人を選択できる。初期選択は [SEN005](../mock-up/sender-entry/sender-entry-select-view-mockup.md) のルールに従う |
| FR-03 | 宛名面レイアウトは要求仕様の項目（差出人エリア・宛名エリア）を印刷できる |
| FR-04 | 郵便番号は横書き、その他主要テキストは縦書き（年賀状一般的配置） |
| FR-05 | 連名は姓省略ルール（`PersonName::join_recipients`）に従う。宛名連名は最大 3 名、差出人連名は最大 4 名（超過時は Validation エラー） |
| FR-06 | プレビュー画面でレイヤー単位の位置オフセットをグラフィカルに調整できる |
| FR-07 | 位置調整は永続化し、次回同種別印刷時にデフォルトとして適用する |
| FR-08 | レイヤーごとの「印刷する / しない」はセッション内のみ有効（永続化しない） |
| FR-09 | 印刷実行前に、はがき種別（年賀状 / 喪中はがき）をダイアログで選択する |
| FR-10 | DB 経路で印刷完了した場合、`PostcardSend` を 1 印刷単位（1 宛名 × 1 差出人 × 1 種別）ごとに作成する |
| FR-11 | 一括印刷では、選択順に 1 ページ 1 宛名の PDF を生成する（複数ページ 1 ジョブ） |
| FR-12 | アーカイブ済み `AddressEntry` / `SenderEntry` は印刷対象に選べない（一覧側で除外またはエラー） |

### 3.2 非機能要件

- オフライン完結（`requirements-and-constraints.md`）
- 印刷エンジン: @react-pdf/renderer（`printing-layout-decision.md`）
- 用紙: 100mm × 148mm、余白 5mm、印刷可能範囲 94mm × 142mm
- 文字サイズ: 6pt 以上
- 差出人は印刷時スナップショットを `PostcardSend` に保存（`sender-domain-v1.md` §6）
- Tauri + SQLite + React の既存レイヤー構成に従う

---

## 4. 現状分析

### 4.1 関連 docs

| ドキュメント | 関連内容 |
|-------------|----------|
| `printing-layout-decision.md` | @react-pdf/renderer、コード内テンプレート、画面内プレビュー |
| `address-domain-v1.md` | AddressEntry、表示名・敬称・連名 |
| `sender-domain-v1.md` | SenderEntry、SenderAddressLink、印刷スナップショット方針 |
| `postcard-receipt-v1-design.md` | 受取は別機能。送付 IF は §5.1.4 で将来予約 |
| `sender-entry-select-view-mockup.md` | 差出人選択 UI・初期選択ルール |

### 4.2 関連実装

| 領域 | 状態 |
|------|------|
| 住所録 CRUD | 実装済（`list/get_address_entry` 等） |
| 差出人 CRUD・リンク | 実装済（`get_sender_id_by_address_entry_id` 等） |
| 受取履歴 | 実装済（TOP-26） |
| 印刷・PDF | **未実装**（@react-pdf/renderer 未導入） |
| 送付情報 | **未実装**（本設計で最小 IF を定義） |

### 4.3 ギャップ

| ギャップ | 対応方針 |
|---------|----------|
| 印刷モジュールなし | フロントに `features/print/` を新設 |
| 要求仕様のレイヤー調整永続化 | TOP-10 の「コード内定義」を**基準座標＋DB オフセット**で拡張 |
| PostcardSend 未設計 | 印刷完了フック用の最小エンティティ・テーブルを本設計で定義 |
| OS 直接印刷 | v1 は PDF 生成＋ OS 印刷ダイアログ（WebView `window.print` または PDF 保存後に OS 既定アプリ）。Tauri ネイティブ印刷 API は非採用 |

---

## 5. 設計

### 5.1 データモデル / DB

#### 5.1.1 印刷用 DTO（アプリ内・非永続）

Tauri コマンドまたはフロント組み立てで、ドメインから以下を生成する。

```typescript
// 宛名 1 件分
type AddressPrintSnapshot = {
  addressEntryId: string;
  postalCode: string;       // formatted "123-4567"
  addressLine1: string;     // prefecture + city + street
  addressLine2: string;     // building（空可）
  primaryLast: string;
  primaryFirst: string;
  coRecipients: { last: string; first: string }[]; // max 3
  honorific: string;
};

type SenderPrintSnapshot = {
  senderEntryId: string;
  postalCode: string;
  addressLine1: string;
  addressLine2: string;
  primaryLast: string;
  primaryFirst: string;
  coRecipients: { last: string; first: string }[]; // max 4
  // 差出人に敬称は要求仕様上なし（印字は氏名＋連名のみ）
};

type PrintJobItem = {
  address: AddressPrintSnapshot;
  sender: SenderPrintSnapshot;
  layerVisibility: Record<PrintLayerId, boolean>; // セッションのみ
  layoutOffsets: Record<PrintLayerId, { dx: number; dy: number }>; // pt。永続化とマージ
};
```

#### 5.1.2 `print_layout_preferences`（永続化）

ユーザーごと（v1 は端末 1 ユーザー想定）・はがき種別ごとに、レイヤー位置オフセットを保存する。

```text
print_layout_preferences
- id                  TEXT PRIMARY KEY
- postcard_type       TEXT NOT NULL          -- 'nengajo' | 'mochu'
- layer_id            TEXT NOT NULL          -- PrintLayerId（§5.3.2）
- offset_x_pt         REAL NOT NULL DEFAULT 0
- offset_y_pt         REAL NOT NULL DEFAULT 0
- updated_at          TEXT NOT NULL

UNIQUE (postcard_type, layer_id)
```

#### 5.1.3 `postcard_sends`（送付情報・最小 v1）

送付機能全設計は別 Issue。印刷完了時の記録に必要な最小スキーマ。

```text
postcard_sends
- id                  TEXT PRIMARY KEY
- address_entry_id    TEXT                    -- FK address_entries。匿名送付は NULL 可（v1 印刷は常に紐付け）
- sender_entry_id     TEXT NOT NULL           -- 参照用 ID
- sender_snapshot     TEXT NOT NULL           -- JSON: SenderPrintSnapshot
- address_snapshot    TEXT NOT NULL           -- JSON: AddressPrintSnapshot（印刷時点）
- postcard_type       TEXT NOT NULL           -- 'nengajo' | 'mochu'
- sent_at             TEXT NOT NULL           -- ISO 8601（印刷実行日時）
- created_at          TEXT NOT NULL
- deleted_at          TEXT                    -- 論理削除（受取と同様 NULL=active）
```

> 送付一覧 UI・編集は別 Issue。本テーブルは印刷フローから `create_postcard_send` で INSERT する。

#### 5.1.4 はがき種別

| 値 | 表示名 | テンプレート |
|----|--------|-------------|
| `nengajo` | 年賀状 | `NengajoAddressTemplate` |
| `mochu` | 喪中はがき | `MochuAddressTemplate` |

v1 ではレイアウト骨格は共通、差異（余白・フォントサイズ等）は種別定数で切替。将来 2 種類以上は拡張。

### 5.2 API / Tauri コマンド

| コマンド | 用途 |
|----------|------|
| `build_address_print_snapshot` | `address_entry_id` → `AddressPrintSnapshot`（archived はエラー） |
| `build_sender_print_snapshot` | `sender_entry_id` → `SenderPrintSnapshot` |
| `resolve_initial_sender_for_address` | SEN005 初期選択ルールを Rust 側で返す |
| `list_print_layout_preferences` | `postcard_type` でオフセット一覧取得 |
| `save_print_layout_preferences` | オフセット一括保存（プレビュー調整確定時） |
| `create_postcard_send` | 印刷完了後に送付 1 件記録 |
| `create_postcard_sends_batch` | 一括印刷完了後に複数 INSERT（トランザクション） |

PDF 生成・プレビューレンダリングは**フロント**（@react-pdf/renderer）で完結。Tauri はデータ取得・永続化・送付記録を担当。

### 5.3 フロントエンド

#### 5.3.1 画面・ルート

| 画面 ID | 名称 | パス（案） | 概要 |
|---------|------|-----------|------|
| PRT001 | 印刷対象選択 | `/print/select` | 住所録一覧から複数選択（既存一覧に「印刷」入口も可） |
| PRT002 | 差出人選択 | `/print/sender` | SEN005 相当。一括時は共通差出人 or 宛名ごとは v1 では**共通 1 差出人**（シンプル化） |
| PRT003 | プレビュー・調整 | `/print/preview` | レイアウトプレビュー、レイヤー調整、種別選択、印刷実行 |
| — | 種別ダイアログ | モーダル | FR-09。印刷直前に表示 |

**一括印刷の差出人方針（v1）**: 全ページで同一 `SenderEntry` を使用。宛名ごとに差出人を変える要件は v1 非スコープ（未決事項へ）。

#### 5.3.2 レイヤー ID（要求仕様準拠）

```
recipient.postalCode
recipient.address1
recipient.address2
recipient.primaryLast
recipient.primaryFirst
recipient.honorific
recipient.coLast.{n}   // n = 1..3
recipient.coFirst.{n}
recipient.coHonorific.{n}  // 連名敬称は宛名全体の honorific を共有
sender.postalCode
sender.address1
sender.address2
sender.primaryLast
sender.primaryFirst
sender.coLast.{n}      // n = 1..4
sender.coFirst.{n}
```

基準座標は `src/features/print/layout/<type>LayoutSpec.ts` に定数定義。実際座標 = 基準 + DB オフセット + セッション調整（未保存分）。

#### 5.3.3 モジュール構成（案）

```
src/features/print/
├── pages/
│   ├── PrintSelectPage.tsx
│   ├── PrintSenderSelectPage.tsx
│   └── PrintPreviewPage.tsx
├── components/
│   ├── PostcardPreviewCanvas.tsx    # 画面プレビュー（HTML/CSS）
│   ├── PrintLayerPanel.tsx          # レイヤー一覧・表示 ON/OFF
│   └── PostcardTypeDialog.tsx
├── pdf/
│   ├── AddressPageDocument.tsx      # @react-pdf/renderer
│   └── renderPrintPdf.ts
├── layout/
│   ├── nengajoLayoutSpec.ts
│   └── mochuLayoutSpec.ts
├── hooks/
│   └── usePrintJob.ts
└── types.ts
```

プレビューは HTML/CSS で近似表示し、PDF 生成時は同一 spec を react-pdf コンポーネントに反映（二重定義を最小化するため spec を共有）。

#### 5.3.4 差出人・宛名の印字ルール

| 項目 | ルール |
|------|--------|
| 宛名氏名 | `primaryLast` + `primaryFirst` + `honorific`（縦書き配置） |
| 宛名連名 | 姓が primary と同じなら姓省略（`join_recipients` 相当を印字用に分解） |
| 差出人氏名 | 姓 + 名（敬称なし） |
| 住所分割 | `addressLine1` = 都道府県+市区町村+町名番地、`addressLine2` = 建物 |
| 郵便番号 | `formatted`、横書き、切手枠・枠外配置 |

Validation（Rust 推奨）:

- 宛名 `coRecipients.len() <= 3`
- 差出人 `coRecipients.len() <= 4`（既存 `SenderEntry::MAX_CO_RECIPIENTS`）

### 5.4 フロー

```mermaid
flowchart TD
  A[住所録一覧 PRT001] -->|1件以上選択| B[差出人選択 PRT002]
  B --> C[プレビュー PRT003]
  C --> D{位置調整}
  D -->|保存| E[print_layout_preferences 更新]
  E --> C
  D -->|印刷| F[種別ダイアログ]
  F --> G[PDF 生成 react-pdf]
  G --> H[OS 印刷 / PDF 保存]
  H --> I[create_postcard_send(s)]
  I --> J[完了サマリ]
```

**一括印刷**: PRT003 でページ送りプレビュー（1/ N）。PDF は N ページ 1 ファイル。送付記録は N 件 INSERT。

**セッション一時データ**: 未保存のレイヤー表示 ON/OFF・未確定オフセットは React state。アプリ終了時に破棄（要求仕様の一時データ削除に該当）。

---

## 6. エッジケース・エラー処理

| ケース | 動作 |
|--------|------|
| 印刷対象 0 件 | 「1 件以上選択してください」 |
| 有効な差出人 0 件 | SEN005 同様、登録導線を表示 |
| 連名上限超過 | Validation エラー。プレビュー入場前にブロック |
| 紐付け差出人が archived | 初期選択スキップ → 最新 updated 差出人を選択 |
| PDF 生成失敗 | エラー表示。送付記録は行わない |
| 印刷キャンセル（OS ダイアログ） | 送付記録は行わない |
| 印刷成功後 DB 失敗 | ユーザーに再試行 or 手動記録を案内（ログ出力） |
| レイアウト prefs 未保存のまま離脱 | 確認ダイアログ |

---

## 7. テスト方針

| 層 | 内容 |
|----|------|
| domain | `AddressPrintSnapshot` 組み立て、連名上限 Validation |
| infrastructure | `print_layout_preferences` CRUD、`postcard_sends` INSERT |
| command_tests | `build_*_snapshot`、archived エラー、batch send |
| frontend | Vitest: spec 座標計算、レイヤー visibility マージ |
| 手動 | 年賀状・喪中のプレビューと PDF 実寸確認、一括 3 件印刷 |

---

## 8. 実装タスク（参考）

| # | タスク | 依存 |
|---|--------|------|
| 1 | `@react-pdf/renderer` 導入、はがき 1 ページ PDF プロトタイプ | — |
| 2 | `nengajoLayoutSpec` + AddressPageDocument | 1 |
| 3 | migration: `print_layout_preferences`, `postcard_sends` | — |
| 4 | Tauri: snapshot 系コマンド + layout prefs CRUD | 3 |
| 5 | PRT001 印刷対象選択（住所録一覧連携） | — |
| 6 | PRT002 差出人選択（SEN005 再利用） | 4 |
| 7 | PRT003 プレビュー + レイヤー調整 UI | 2, 4 |
| 8 | 種別ダイアログ + PDF 出力 + OS 印刷 | 7 |
| 9 | `create_postcard_send(s)` 連携 | 4, 8 |
| 10 | 喪中テンプレート差分 | 2 |
| 11 | 一括印刷（多ページ PDF + batch send） | 8, 9 |

---

## 9. 未決事項

| 項目 | 内容 | 扱い |
|------|------|------|
| 一括印刷時の差出人 | v1 は全ページ共通 1 差出人。宛名ごと変更は将来 | 本設計で v1 は共通に固定 |
| CSV 印刷経路 | 要求仕様に記載。v1.1 CSV と合わせて設計 | 非スコープ |
| OS 印刷の具体 API | `window.print` vs PDF 保存後シェル open | 実装 Issue（TOP-28）で Windows 優先検証 |
| 連名敬称の個別レイヤー | 要求仕様は連名敬称 n あり。v1 は全体 `honorific` 1 つを各連名に適用 | 実装時に UI 簡略化 |
| PostcardSend 詳細設計 | 一覧・編集・受取連携 | 送付情報管理 Issue で拡張 |
| @react-pdf 縦書き | ライブラリ制約次第で html2canvas 併用 | 実装時スパイク |

---

## 10. 自己レビュー記録（設計時）

```
- [x] Issue 要件の充足（ブロッカー）— TOP-19 スコープ 5 項目を §2–§5 でカバー
- [x] 機能矛盾なし（ブロッカー）— 受取/送付/印刷の責務分離を明記
- [x] 実装済み機能との整合（ブロッカー）— Address/Sender 既存 API・ドメインルールを参照
- [x] 方針・要件・アーキテクチャとの整合（ブロッカー）— TOP-10 react-pdf、オフライン、Tauri+SQLite
- [x] 改善点は対応済み、または未決事項に移した — §9 参照
```
