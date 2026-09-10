# はがき送付情報管理（v1）— 設計書

- **Linear Issue**: [TOP-18](https://linear.app/topea-3/issue/TOP-18/v1-はがき送付情報管理の機能設計) v1 はがき送付情報管理の機能設計
- **親 Issue**: [TOP-14](https://linear.app/topea-3/issue/TOP-14/v1-機能設計) v1 機能設計
- **後続実装 Issue**: [TOP-27](https://linear.app/topea-3/issue/TOP-27/v1-はがき送付情報管理機能の実装)
- **関連設計**: [postcard-address-print-v1-design.md](./postcard-address-print-v1-design.md)（`PostcardSend` 最小定義・印刷時作成）、[postcard-receipt-v1-design.md](./postcard-receipt-v1-design.md)（受取連携 IF）
- **ステータス**: Reviewed
- **最終更新**: 2026-09-10

---

## 1. 背景・目的

v1.0.0 では、年賀状等の**送付事実**を記録し、「今年送った / 送っていない」を把握できるようにする。印刷（TOP-19 / TOP-28）は PDF 成功時に `PostcardSend` を作成する経路を既に持つ。本設計は **一覧・検索・手動登録・編集・論理削除・送付状況判定・一括登録方針** を実装可能な粒度まで定義する。

---

## 2. スコープ

### 2.1 対象（v1 / TOP-18）

- 送付履歴エンティティ `PostcardSend` のドメイン拡張（印刷最小定義の上に CRUD・照会を載せる）
- 住所録・差出人・受取履歴との関係整理
- 送付履歴一覧（年度・種別・検索）と詳細・編集・論理削除
- 「指定年・種別に送った / 送っていない」判定ロジックと送付状況ビュー
- 単件登録・一括登録のインターフェース方針（印刷経路との役割分担含む）

### 2.2 非スコープ

- 宛名面印刷フロー本体（PRT001–003。送付記録の**作成側**は印刷設計済み）
- 暑中・寒中など `nenga` / `mochu` 以外の種別プリセット（v1.1 以降。Issue 文言の「ほか」は将来拡張）
- CSV 入出力・バックアップ/リストア（v1.1 以降）
- 削除済み一覧・復元 UI
- 複数ユーザー・端末間同期
- ユーザー定義テンプレート CRUD（コード内テンプレート方針のまま。`postcard_type` がテンプレート識別子）

---

## 3. 要件

### 3.1 機能要件

| ID | 要件 |
|----|------|
| FR-01 | ユーザーは送付履歴 1 件を「宛先（住所録）」「差出人」「送付日」「種別」「スナップショット」として参照できる |
| FR-02 | 印刷フロー（既存 `create_postcard_sends_batch`）で作成された履歴を一覧・詳細で確認できる |
| FR-03 | アプリ外で送付した場合など、印刷を経ずに手動で送付履歴を登録できる（単件） |
| FR-04 | 複数宛先を選び、同一種別・送付日で一括登録できる（上限 200。印刷と同じ） |
| FR-05 | 一覧で年度・種別でフィルタし、フリーテキスト検索（宛名・差出人ラベル相当）ができる |
| FR-06 | 指定年・種別について「送った住所録」「送っていない住所録」を一覧できる |
| FR-07 | 受取履歴（指定年）に紐づく住所録を送付候補として参照できる（任意フィルタ） |
| FR-08 | 詳細表示・送付日/種別/メモの編集・論理削除ができる。宛先・差出人の差し替えは v1 非対応（削除＋再登録） |
| FR-09 | デフォルトソートは送付日降順 |
| FR-10 | 使用テンプレートは v1 では `postcard_type`（`nenga` / `mochu`）と同一視する |

### 3.2 非機能要件

- オフライン完結（`requirements-and-constraints.md`）
- 個人利用規模（数千件）で一覧・送付状況照会が実用的な応答時間
- 送付履歴は **`deleted_at IS NULL` を active 条件**（受取履歴と同方針。住所録の `archived_at` とは別）
- 年度は `sent_on`（`YYYY-MM-DD`）の暦年。端末ローカル日付規約は印刷・受取と同一
- Tauri コマンド + SQLite（sqlx）+ React の既存レイヤー構成に従う
- 既存 `postcard_sends` テーブル・印刷用 `create_postcard_sends_batch` と破壊的に矛盾しない

---

## 4. 現状分析

### 4.1 関連 docs

| ドキュメント | 関連内容 |
|-------------|----------|
| `docs/overview/decisions-summary.md` | History ラフ（送受信一体）→ v1 では受取/送付に分割 |
| `docs/design/postcard-address-print-v1-design.md` | `postcard_sends` 最小スキーマ、スナップショット、batch 作成、idempotency |
| `docs/design/postcard-receipt-v1-design.md` | 受取 CRUD、§5.1.4 送付候補抽出 IF |
| `docs/domain/address-domain-v1.md` | AddressEntry、archive |
| `docs/domain/sender-domain-v1.md` | SenderEntry、SenderAddressLink |

### 4.2 関連実装

| 領域 | 状態 |
|------|------|
| `postcard_sends` テーブル（migration 0006） | 実装済 |
| `PostcardSend` domain / `create_batch` / `create_postcard_sends_batch` | 実装済（印刷専用） |
| 印刷 UI からの送付記録 | 実装済 |
| 送付 CRUD・一覧・送付状況 API / UI | **未実装** |
| 受取履歴 CRUD | 実装済 |

### 4.3 ギャップ

| ギャップ | 対応方針 |
|---------|----------|
| 印刷は INSERT のみ。search / get / update / delete なし | repository・コマンドを拡張 |
| 手動・一括登録経路なし | `create_postcard_sends_manual_batch` を追加（印刷 batch とは入力・検証が異なる） |
| メモ・登録経路（印刷/手動）の区別なし | migration で `memo` / `source` を追加 |
| 「未送付」は住所録 × 送付の差集合 | `search_send_status` 系コマンドを新設 |
| Issue の暑中/寒中 | v1 非スコープ（既存 CHECK `nenga`/`mochu` を維持） |

---

## 5. 設計

### 5.1 ドメイン

#### 5.1.1 エンティティ: `PostcardSend`（拡張）

**役割**: はがきを 1 通送付した事実を 1 件表す。印刷完了時および手動登録の両方で同一エンティティを用いる。

| 属性 | 型 | 必須 | 説明 |
|------|-----|------|------|
| `id` | UUID | ○ | 主キー |
| `printJobId` | UUID | ○ | 同一バッチの冪等キー。印刷ジョブ ID、または手動一括のバッチ UUID |
| `addressEntryId` | UUID | ○ | 宛先住所録（必須。匿名送付は v1 非対応） |
| `senderEntryId` | UUID | ○ | 差出人 |
| `senderSnapshot` | JSON string | ○ | 登録時点の差出人印字スナップショット |
| `addressSnapshot` | JSON string | ○ | 登録時点の宛名印字スナップショット |
| `postcardType` | `PostcardType` | ○ | `nenga` \| `mochu`（＝使用テンプレート識別） |
| `sentOn` | date | ○ | 送付日（暦日） |
| `source` | `PostcardSendSource` | ○ | `print` \| `manual` |
| `memo` | string \| null | — | 自由記述（最大 1000 文字想定） |
| `deletedAt` | datetime \| null | — | null = 有効 |
| `createdAt` / `updatedAt` | datetime | ○ | 監査用 |

**ドメインルール**

- `sentOn` は未来日不可（手動作成・更新時 Validation）。印刷経路は従来どおりコマンド内 `Local::today` のため未来にならない
- create/update 時、`addressEntryId` / `senderEntryId` は **active**（`archived_at IS NULL`）のみ許可
- 削除は `deletedAt` 論理削除。v1 に復元 UI なし
- 更新で変更可能なのは `sentOn` / `postcardType` / `memo` のみ。スナップショット・宛先・差出人・`printJobId`・`source` は不変
- 同一 `(printJobId, addressEntryId)` の active 行は UNIQUE（既存）。意図した再送付は別バッチ UUID で別レコード

#### 5.1.2 値オブジェクト

**`PostcardType`**（既存・印刷と共有）

| 値 | 表示名 | 印刷テンプレート |
|----|--------|------------------|
| `nenga` | 年賀状 | あり |
| `mochu` | 喪中はがき | あり |

**`PostcardSendSource`**

| 値 | 表示名 |
|----|--------|
| `print` | 印刷 |
| `manual` | 手入力 |

#### 5.1.3 関係整理

```
AddressEntry 1 ──< N PostcardSend
SenderEntry  1 ──< N PostcardSend
AddressEntry 0..1 ── SenderAddressLink ── 1 SenderEntry   （手動登録時のデフォルト差出人解決）
AddressEntry 1 ──< N PostcardReceipt                     （送付候補の参照元。送付行への FK は持たない）
```

| 関係 | 方針 |
|------|------|
| 住所録 | 必須 FK。一覧表示はスナップショット優先、必要なら live JOIN で補完 |
| 差出人 | 必須 FK。印刷と同じく登録時スナップショット保存 |
| 受取 | **FK なし**。候補抽出は `address_entry_id` + 年のクエリ結合 |
| テンプレート | DB 上の別テーブルなし。`postcard_type` がコード内テンプレートキー |

**採用理由**: 印刷設計が既にスナップショット付き `postcard_sends` を導入済みのため、別 History テーブルを立てず同一実体を拡張する。受取との自動 1:1 紐付けは過剰（1 相手に複数受取・複数送付がありうる）。

#### 5.1.4 「今年送った / 送っていない」判定

**定義（パラメータ化）**

- 入力: `year`（必須）、`postcardType`（任意。null = 種別不問）、`receiptYear`（任意・候補絞り込み）
- 「送った」: active な `AddressEntry` のうち、active な `PostcardSend` が  
  `sent_on ∈ [year-01-01, year-12-31]` かつ（指定時）`postcard_type = ?` を **1 件以上**持つ
- 「送っていない」: active な `AddressEntry` のうち、上記を満たす送付が **0 件**
- 「今年」UI デフォルト: `PostcardSend::local_today().year()`（端末ローカル）

**重複送付**: 同一年・同一種別に複数 `PostcardSend` があっても「送った」は 1 回とみなす（status 集約）。履歴一覧では複数行をそのまま出す。

**受取連携（任意）**

- `receiptYear` 指定時: 候補母集団を  
  `postcard_receipts`（`deleted_at IS NULL` AND `address_entry_id IS NOT NULL` AND 受取日がその年）の distinct `address_entry_id` に限定  
  （`postcard-receipt-v1-design.md` §5.1.4 の実現）

```mermaid
flowchart LR
  A[active AddressEntry] --> B{指定年・種別に<br/>active PostcardSend あり?}
  B -->|Yes| C[sent]
  B -->|No| D[unsent]
  E[任意: 指定年の受取あり] -.->|母集団を絞る| A
```

### 5.2 データモデル / DB

#### 5.2.1 既存テーブル（変更なしの列）

`postcard_sends`（migration 0006）を継続利用。既存列の意味は印刷設計どおり。

#### 5.2.2 追加マイグレーション（案: `0007_alter_postcard_sends_for_manage.sql`）

```sql
ALTER TABLE postcard_sends ADD COLUMN source TEXT NOT NULL DEFAULT 'print'
  CHECK (source IN ('print', 'manual'));

ALTER TABLE postcard_sends ADD COLUMN memo TEXT;
```

- 既存行は `source = 'print'`（DEFAULT）
- `memo` は NULL 可。アプリ Validation で最大長

#### 5.2.3 一覧・送付状況クエリ方針

| 用途 | SQL 概要 |
|------|----------|
| 履歴 active | `deleted_at IS NULL` |
| 年度 | `sent_on` が `YYYY-01-01`〜`YYYY-12-31` |
| 種別 | `postcard_type = ?` |
| 検索 | スナップショット JSON 内氏名、または JOIN `address_entries` / `sender_entries` の表示名 LIKE |
| ソート | `sent_on DESC, id ASC` |
| ページング | `LIMIT` / `OFFSET`（`MAX_PAGE_LIMIT = 200`） |
| 送付済 ID | `SELECT DISTINCT address_entry_id FROM postcard_sends WHERE ... year/type ...` |
| 未送付 | active address LEFT ANTI JOIN 上記、または `NOT IN` / `NOT EXISTS` |

検索の正: v1 は **live の AddressEntry / SenderEntry 表示名 + memo** を優先し、アーカイブ済みで JOIN できない場合はスナップショットの氏名にフォールバック（受取一覧と同系統）。

### 5.3 API / Tauri コマンド

命名・エラー処理は住所録・受取・印刷に準拠。

| コマンド | 概要 |
|----------|------|
| `create_postcard_sends_batch` | **既存**。印刷専用。`source=print`、`sent_on=Local::today`、メモなし |
| `create_postcard_sends_manual_batch` | 手動単件/一括。入力の `sent_on` / `postcard_type` / 任意 `memo`。各宛先について差出人解決→スナップショット生成→INSERT。バッチ UUID をサーバ発行 |
| `get_postcard_send` | 詳細 1 件（スナップショット + live 補完表示用フィールド） |
| `search_postcard_sends` | 履歴一覧（`items` + `total`） |
| `list_postcard_send_years` | フィルタ用の年一覧（受取の `list_postcard_receipt_years` と同様） |
| `update_postcard_send` | `sent_on` / `postcard_type` / `memo` のみ。`expected_updated_at` による楽観ロック（受取更新に準拠） |
| `delete_postcard_send` | 論理削除 |
| `search_send_status` | 送付状況（住所録ベース、`sent` \| `unsent`） |

#### `create_postcard_sends_manual_batch` 入力（案）

```typescript
{
  postcardType: 'nenga' | 'mochu';
  sentOn: string;                 // 'YYYY-MM-DD'
  memo?: string | null;           // 全件共通メモ（v1）
  items: {
    addressEntryId: string;
    senderEntryId?: string | null; // 省略時は SenderAddressLink から解決。無ければ Validation
  }[];                            // 1〜200
}
```

**差出人解決順（各 item）**

1. `senderEntryId` 指定 → active 検証
2. 未指定 → `get_sender_id_by_address_entry_id` → リンク無し/archived は当該 item をエラー理由付きで失敗（**all-or-nothing**）
3. スナップショットは印刷と同じ `build_*_print_snapshot` 経路を再利用（INSERT 時に再取得しない／取得結果をそのまま保存）

バッチ UUID: コマンド内で 1 つ発行し、全行の `print_job_id` に設定。`source = manual`。UNIQUE 衝突は通常発生しない（新規 UUID）。

#### `search_postcard_sends` 入力（案）

```typescript
{
  keyword?: string | null;
  year?: number | null;
  postcardType?: string | null;
  addressEntryId?: string | null;
  source?: 'print' | 'manual' | null;
  includeDeleted?: boolean;       // default false
  limit: number;
  offset: number;
  sortOrder: 'desc' | 'asc';
}
```

#### `search_send_status` 入力（案）

```typescript
{
  year: number;                   // 必須
  postcardType?: string | null;   // null = 種別不問
  status: 'sent' | 'unsent';
  receiptYear?: number | null;    // 指定時は受取ありの住所録に限定
  keyword?: string | null;        // 住所録表示名検索
  limit: number;
  offset: number;
}
```

応答: `items: { addressEntryId, displayName, addressSummary, lastSentOn?, sendCount }[]` + `total`。

#### Validation エラー例

| 条件 | メッセージ（案） |
|------|------------------|
| 未来の送付日 | 「送付日に未来の日付は指定できません。」 |
| 宛先 not found / archived | `address entry not found` / `address entry is archived` |
| 差出人解決不可 | 「差出人が紐づいていない宛名があります。」（ID 一覧付き可） |
| 不正な種別 | 種別の Validation エラー |
| 件数 0 または 200 超 | 件数 Validation |

### 5.4 フロントエンド

#### 5.4.1 画面一覧

| 画面 ID | 名称 | パス（案） | モック |
|---------|------|-----------|--------|
| SND001 | 送付管理（履歴タブ） | `/sends` | `docs/mock-up/postcard-send/postcard-send-list-view-mockup.md` |
| SND002 | 送付履歴作成（単件） | `/sends/new` | create |
| SND003 | 送付履歴編集 | `/sends/:id/edit` | edit |
| SND004 | 送付履歴詳細 | `/sends/:id` | detail |
| SND005 | 送付管理（送付状況タブ） | `/sends?tab=status` または `/sends/status` | status |
| SND006 | 一括登録 | `/sends/bulk` | bulk |

ナビゲーション: 共通ヘッダーに「送付履歴」を追加（受取・住所録・差出人と並列）。

**SND001 / SND005 は同一シェルのタブ**とする（Issue の「一覧で年度・種別・送付有無・検索」を 1 画面族で満たす）。

| タブ | 主エンティティ | 送付有無 |
|------|----------------|----------|
| 履歴 | `PostcardSend` 行 | なし（送付済み事実の一覧） |
| 送付状況 | `AddressEntry` 行 | **あり**（送った / 送っていない） |

#### 5.4.2 履歴タブ（SND001）要点

- フィルタ: 送付年、種別、登録経路（任意）
- 検索: 宛名・差出人・メモ
- カラム: 送付日 / 種別 / 宛名 / 差出人 / 経路 / メモ抜粋 / 操作
- 行クリック → SND004
- ヘッダ: 「新規作成」「一括登録」+ タブ切替

#### 5.4.3 作成・一括

| 画面 | 要点 |
|------|------|
| SND002 | 宛名選択 → 差出人（リンクデフォルト、手動変更可）→ 送付日・種別・メモ |
| SND006 | 住所録複数選択（最大 200）→ 共通の送付日・種別・メモ → 差出人は原則リンク自動。未紐付けは事前チェックでブロックまたは除外確認 |

印刷フローとの分担: **これから印刷して送る** → PRT001 起点。**既に送った事実を記録** → SND002 / SND006。

#### 5.4.4 送付状況タブ（SND005）

- 必須: 対象年、種別（「すべて」可）、表示: 送った / 送っていない（＝送付有無フィルタ）
- 任意: 「この年に受取ありの相手に限定」チェック（`receiptYear`、デフォルトは対象年と同じ）
- 検索: 住所録表示名
- 未送付行から「印刷へ」（選択 ID を `printJobDraft` へ）または「一括登録へ」の導線

### 5.5 主要ユースケース

```mermaid
flowchart TD
  A[送付する] --> B{アプリで印刷する?}
  B -->|Yes| C[PRT001〜003 → create_postcard_sends_batch]
  B -->|No| D{件数}
  D -->|1| E[SND002 手動登録]
  D -->|複数| F[SND006 一括登録]
  C --> G[SND001 で確認]
  E --> G
  F --> G
  G --> H[SND005 で未送付を確認]
  H --> I[受取フィルタで候補絞り込み]
  I --> J[印刷 or 一括登録へ]
```

```mermaid
sequenceDiagram
  participant UI as SND006
  participant Cmd as create_postcard_sends_manual_batch
  participant Link as SenderAddressLink
  participant Snap as build_*_snapshot
  participant DB as postcard_sends

  UI->>Cmd: type, sentOn, items[]
  Cmd->>Cmd: batch UUID 発行
  loop each item
    Cmd->>Link: sender 解決
    Cmd->>Snap: address/sender snapshot
  end
  Cmd->>DB: INSERT source=manual（1 TX）
  Cmd-->>UI: ok
```

---

## 6. エッジケース・エラー処理

| ケース | 方針 |
|--------|------|
| 宛先/差出人を後からアーカイブ | 履歴は残る。一覧はスナップショット表示。live リンクは「（アーカイブ済み）」 |
| 同一年・同一相手・同一種別の複数送付 | 許可（再送・誤記録修正前など）。status は「送った」 |
| 印刷の誤記録 | SND001 から論理削除（印刷設計の方針どおり） |
| 手動一括で 1 件でも差出人なし | all-or-nothing。失敗理由に addressEntryId を含める |
| タイムゾーンと年度 | `sent_on` は日付文字列のみ。年フィルタは文字列範囲比較 |
| 空の送付状況 | 空状態 + 印刷/登録導線 |
| `update` で種別変更 | スナップショットは印刷レイアウトと一致しなくなる可能性あり。許容（履歴の事実修正）。再印刷は別レコード |

---

## 7. テスト方針

| 層 | 内容 |
|----|------|
| domain | 未来日 NG、`PostcardSendSource`、更新で不変フィールドを変えないこと |
| repository | CRUD、年/種別/keyword、`search_send_status` の sent/unsent、receiptYear 絞り込み、`deleted_at` |
| command | 手動 batch の差出人解決失敗、200 超、印刷 batch との共存（source） |
| 結合 | migration 0007 適用後の command_tests |
| フロント | 一覧フィルタ、SND005 ↔ 印刷/一括導線（モック準拠の結合テスト） |

---

## 8. 実装タスク（TOP-27 向け）

- [ ] migration `0007_alter_postcard_sends_for_manage.sql`（`source` / `memo`）
- [ ] domain: `PostcardSend` 拡張、`PostcardSendSource`、update 用ファクトリ
- [ ] repository: get / search / update / soft_delete / years / send_status
- [ ] 既存 `create_batch` が `source=print` を明示するよう調整
- [ ] Tauri: `create_postcard_sends_manual_batch` ほか CRUD・`search_send_status`
- [ ] frontend: `features/postcard-send/`、SND001–006
- [ ] ルーティング・ナビゲーション
- [ ] AddressEntry 複数選択 UI の再利用（印刷 PRT001 / 受取ダイアログ）
- [x] mock-up（設計フェーズで作成）

---

## 9. 未決事項

| 項目 | 状態 | 備考 |
|------|------|------|
| 暑中・寒中プリセット | v1 非採用 | 必要なら v1.1 で `PostcardType` 拡張＋印刷テンプレ有無を再設計 |
| 一括登録のメモを行ごとに変える | v1 非対応 | 全件共通メモのみ |
| 送付状況から印刷へ渡す draft の詳細キー | 実装時 | 既存 `printJobDraft` 形式に合わせる |
| 検索をスナップショット JSON 直叩きにするか | 実装判断 | 設計は live JOIN 優先を推奨 |

---

## 変更履歴

| 日付 | 内容 |
|------|------|
| 2026-09-10 | 初版（TOP-18 設計） |
| 2026-09-10 | 自己レビュー: 履歴/送付状況を同一シェルのタブに整理、update に楽観ロックを明記 |
