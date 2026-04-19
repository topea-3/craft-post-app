# Craft Post App — 差出人情報ドメイン（v1）

## 1. ドメインオブジェクト全体像

- **エンティティ**
  - `SenderEntry`（差出人エントリ）
  - `SenderAddressLink`（差出人と宛名住所の紐づけ）
- **値オブジェクト**
  - `SenderLabel`（差出人ラベル）
  - `PersonName`（氏名）
  - `PostalCode`（郵便番号）
  - `Address`（住所本体）
  - `PhoneNumber`（電話番号）

> v1 では「差出人＝自宅/会社など複数パターン」を扱えるようにしつつ、  
> 印刷・送付フローから **確実に選択できる** ことを最優先とする。

将来機能（ユーザー切替、履歴 `PostcardHistory` からの参照、バックアップ/リストアなど）を見据え、  
`SenderEntry` を中心に拡張しやすい形を意識して設計する。

---

## 2. エンティティ

### 2.1 SenderEntry（差出人エントリ）

**役割**: はがきの差出人欄に印字する情報を表す。  
**識別子**: 永続化層の主キー `id`（UUID）によって一意に識別される。

- **属性構成（論理モデル）**
  - `id` : 内部 ID（UUID。SQLite では TEXT カラムで保持）
  - `label` : 差出人ラベルを表す `SenderLabel` 値オブジェクト（例: `"自宅"`, `"会社"`）
  - `primaryName` : 主たる氏名を表す `PersonName` 値オブジェクト
  - `coRecipients` : 連名用の `PersonName` 値オブジェクトの配列（0 件以上）
  - `postalCode` : `PostalCode` 値オブジェクト
  - `address` : `Address` 値オブジェクト
  - `phoneNumber` : `PhoneNumber` 値オブジェクト（任意）
  - `archivedAt` : アーカイブ日時（任意）。未設定（`null`）＝有効、設定済み＝論理削除（アーカイブ）済み。
  - `createdAt` : 作成日時
  - `updatedAt` : 更新日時
- **主なドメインルール（v1）**
  - `label` / `primaryName` / `postalCode` / `address` は必須（空の差出人は作らない）。
  - 連名は `coRecipients` として 0〜4 件保持できる（例: 夫婦連名、家族連名）。
  - 差出人欄の表示名は、`primaryName` と `coRecipients` を結合した表示名とする
    - 結合ルールは `docs/domain/address-domain-v1.md` の `joinRecipients(primaryName, coRecipients)` に準拠する（同姓省略など）。
  - `archivedAt` が設定されているエントリは通常の一覧・選択 UI からは非表示とする（論理削除）。
  - `label` は **`archivedAt` が未設定のエントリ同士の範囲で一意** とする（選択 UI の識別子として使うため）。
  - 宛名住所との関連は **`SenderAddressLink` 経由**とする（`SenderEntry` 単体に宛名FKは持たない）。

### 2.2 SenderAddressLink（差出人↔宛名住所 紐づけ）

**役割**: `SenderEntry` と `AddressEntry` を結びつける。  
**識別子**: 永続化層の主キー `id`（UUID）。

- **属性構成（論理モデル）**
  - `id` : 内部 ID（UUID）
  - `senderEntryId` : `SenderEntry.id`
  - `addressEntryId` : `AddressEntry.id`
  - `createdAt` / `updatedAt`
- **主なドメインルール（v1）**
  - **1 つの `SenderEntry` に対して、複数の `AddressEntry` を紐づけ可能**（差出人1・宛名複数）。
  - **1 つの `AddressEntry` に対しては、差出人候補を複数持たない**  
    → 有効なリンクの範囲では、**同一 `addressEntryId` は高々 1 行**（どの差出人に紐づくかは一意）。アプリ層で担保する（DBにUNIQUEは作らない方針）。
  - 同一 `(senderEntryId, addressEntryId)` のリンクは **高々 1 件**（アプリ層で担保）。
  - 上記により、実質のカーディナリティは **差出人 1 : 宛名 N**、**宛名 1 : 差出人 0..1**（紐づきがある場合）として扱える。

---

## 3. 値オブジェクト

### 3.1 SenderLabel（差出人ラベル）

**役割**: ユーザーが差出人を識別・選択するための名称（例: `"自宅"`, `"会社"`）。

- **属性**
  - `value` : 本文
- **不変条件（例）**
  - 非空
  - 最大文字数は **250 文字**（説明用途を許容する）

---

### 3.2 PersonName（氏名）

`docs/domain/address-domain-v1.md` の `PersonName` と同一の設計を採用する。

---

### 3.3 PostalCode（郵便番号）

`docs/domain/address-domain-v1.md` の `PostalCode` と同一の設計を採用する。

---

### 3.4 Address（住所本体）

`docs/domain/address-domain-v1.md` の `Address` と同一の設計を採用する。

- `prefecture`（必須・プリセット選択）
- `city`（必須）
- `street`（必須）
- `building`（任意）

---

### 3.5 PhoneNumber（電話番号）

**役割**: 差出人の連絡先電話番号。

- **属性**
  - `value` : 本文（表示用を兼ねる）
- **不変条件（v1 案）**
  - 任意入力（未入力可）
  - 許容形式は緩め（ハイフン有無どちらも許容）。保存時に数字のみ正規化するかは実装で判断。

---

## 4. 一覧・選択画面とドメインの対応

### 4.1 差出人一覧（SenderEntry List View）

- **表示対象**
  - `archivedAt` が未設定（有効）の `SenderEntry` を対象とする。
- **想定表示項目**
  - ラベル: `SenderLabel.value`
  - 氏名（連名対応）:
    - 単名: `primaryName.display`
    - 連名: `joinRecipients(primaryName, coRecipients)`
  - 郵便番号: `PostalCode.formatted`
  - 住所: `Address.toSingleLine()`
  - 最終更新日時: `updatedAt`
- **ソート（v1 確定）**
  - 更新日（`updatedAt`）**降順のみ**（固定）
  - 検索・オプションソートは v1 では提供しない（個人利用で差出人が大量に増えない想定のため）

---

### 4.2 差出人選択（印刷/送付フロー）

- **初期選択（宛名住所と紐づいている場合）**
  - 対象の `AddressEntry.id` に一致する `SenderAddressLink` が 1 件あり、紐づいた `SenderEntry` の `archivedAt` が未設定（有効）の場合:
    - その `SenderEntry` を選択
- **初期選択（紐づきが無い／候補が無い場合）**
  - `updatedAt` が最新のものを選択
- **未登録時**
  - 「差出人を登録してください」を表示し、作成導線を提供

---

## 5. ライフサイクルとユースケース（Create/Update/Delete）

### 5.1 作成（CreateSenderEntry）

- 入力フォームから値を受け取り、各値オブジェクトの不変条件に従って `SenderEntry` を生成。

### 5.2 更新（UpdateSenderEntry）

- 既存の `SenderEntry` をロードし、編集フォームに展開。
- 変更後の入力で再度値オブジェクトを構築・検証し、問題なければ保存。
  - 宛名との紐づけは `SenderAddressLink` の追加・削除で行う（差出人1件に宛名を複数紐づけ可能）。
  - 新規リンク追加時、対象 `addressEntryId` が **既に別の差出人に紐づいている**場合は確認ダイアログで置き換えを確認し、OKの場合は既存リンクを削除して付け替える。

### 5.3 削除（ArchiveSenderEntry）

- v1 の基本方針として **論理削除（アーカイブ）** を採用:
  - `archivedAt` にアーカイブ実行日時を書き込む `ArchiveSenderEntry` ユースケースを主とする（`updatedAt` は変更しない）。
- 差出人（`SenderEntry`）をアーカイブした場合、その差出人に紐づく宛名とのリンクは削除する。

---

## 6. 印刷機能との連携方針（IF）

### 6.1 方針: スナップショット保存

- 印刷ジョブ/プレビュー作成時に `SenderEntry` を **スナップショット** して保存する。
  - `senderId`（参照用）
  - `senderSnapshot`（印刷に使った実値）
- 理由: 差出人編集により過去の印刷結果が変わる事故を防ぐ。

---

## 7. テーブル設計（SQLite）

差出人ドメインを SQLite 上のテーブルとして表現するための設計（v1 想定）を示す。

### 7.1 `sender_entries` テーブル（差出人エントリ本体）

**役割**: `SenderEntry` エンティティに対応。差出人のラベル・印字名・住所など中心情報を保持する。

```text
sender_entries
- id                TEXT PRIMARY KEY                       -- 内部ID（UUID文字列）

- label             TEXT    NOT NULL                       -- 差出人ラベル（非アーカイブ内で一意、最大 250 文字）
- primary_last      TEXT    NOT NULL                       -- 主たる氏名 姓
- primary_first     TEXT    NOT NULL                       -- 主たる氏名 名
- primary_kana_last TEXT                                  -- 主たる氏名 カナ姓
- primary_kana_first TEXT                                 -- 主たる氏名 カナ名

- postal_code       TEXT    NOT NULL                       -- 7 桁（"1234567"）想定

- prefecture        TEXT    NOT NULL                       -- 都道府県（プリセットから選択）
- city              TEXT    NOT NULL                       -- 市区町村
- street            TEXT    NOT NULL                       -- 町名・番地
- building          TEXT                                   -- 建物名・部屋番号

- phone_number      TEXT                                   -- 電話番号（任意）

- archived_at       TEXT                                   -- アーカイブ日時（ISO 8601）。NULL = 有効

- created_at        TEXT    NOT NULL                       -- ISO 8601 形式
- updated_at        TEXT    NOT NULL                       -- ISO 8601 形式
```

**インデックス案**

```text
CREATE INDEX idx_sender_entries_active_updated_at
    ON sender_entries (updated_at DESC) WHERE archived_at IS NULL;
```

> 注: 論理削除運用のため、ユニーク制約（UNIQUE INDEX 等）は原則 **DBには作らず**、アプリケーション側のチェックで担保する（例: `label` の重複）。

---

### 7.2 `sender_co_recipients` テーブル（連名者）

**役割**: `SenderEntry.coRecipients: PersonName[]` に対応。1 差出人に対する 0 件以上の連名者を保持する。
（v1 では **最大 4 件** とする。DB での CHECK 制約よりもアプリケーション層での制約を優先し、UI/ユースケースで弾く。）

```text
sender_co_recipients
- id                TEXT PRIMARY KEY                       -- 内部ID（UUID文字列）
- sender_entry_id   TEXT NOT NULL                          -- FK -> sender_entries.id（UUID）
- order_index       INTEGER NOT NULL                        -- 表示順（0,1,2,...）

- last              TEXT    NOT NULL                        -- 姓
- first             TEXT    NOT NULL                        -- 名
- kana_last         TEXT                                    -- カナ姓
- kana_first        TEXT                                    -- カナ名

- archived_at       TEXT                                     -- アーカイブ日時（ISO 8601）。NULL = 有効
- created_at        TEXT    NOT NULL                        -- ISO 8601 形式
- updated_at        TEXT    NOT NULL                        -- ISO 8601 形式
```

**インデックス案**

```text
CREATE INDEX idx_sender_co_recipients_entry_order
    ON sender_co_recipients (sender_entry_id, order_index);
```

---

### 7.3 `sender_address_links` テーブル（差出人↔宛名住所 紐づけ）

**役割**: `SenderAddressLink` に対応。**差出人1に宛名複数**を表現する。宛名側は **1宛名あたり紐づく差出人は高々1件**。

```text
sender_address_links
- id                TEXT PRIMARY KEY                       -- 内部ID（UUID文字列）

- sender_entry_id   TEXT NOT NULL                          -- FK -> sender_entries.id
- address_entry_id  TEXT NOT NULL                          -- FK -> address_entries.id

- created_at        TEXT    NOT NULL                       -- ISO 8601 形式
- updated_at        TEXT    NOT NULL                       -- ISO 8601 形式
```

**インデックス案**

```text
CREATE INDEX idx_sender_address_links_address
    ON sender_address_links (address_entry_id);

CREATE INDEX idx_sender_address_links_sender
    ON sender_address_links (sender_entry_id);
```

> 注: `(sender_entry_id, address_entry_id)` の重複に加え、**同一 `address_entry_id` は有効データとして高々1件**（宛名あたり差出人は1人まで）を **アプリ層で担保**（DBにUNIQUEは作らない方針）。

---

## 8. 未決事項（要決定）

1. （確定）電話番号の正規化はしない
2. **連名の入力上限**: 最大 4 件（確定）
3. **宛名に紐づきが無い／紐づいた差出人がアーカイブ済み（`archivedAt` 設定済み）の場合の初期選択**: 全体から `updatedAt` 最新を初期選択する（ただし自動的な紐づけはしない）※選択ルールの確定は要確認
4. （確定）紐づけの管理UXは、両方向から紐づけ可能とする（差出人編集画面で宛名を選択／宛名側で差出人を紐づけ）
---

## 変更履歴

| 日付       | 内容 |
|------------|------|
| 2026-03-18 | 初版。TOP-16 のたたき台として、差出人ドメインをエンティティ・値オブジェクト・テーブル設計で整理。 |
| 2026-03-18 | 宛名との関連を **`SenderAddressLink`（リンクテーブル）** で表現し、**差出人1に宛名複数**を主たる意図として明記。 |
| 2026-03-18 | **1宛名あたり紐づく差出人は高々1件**に制限。`preferred` を廃止。 |
| 2026-04-19 | 論理削除を `archived`（INTEGER）から `archived_at`（TEXT, NULL = 有効）へ変更。ラベル一意は未アーカイブ行に限定。アーカイブ時は `updated_at` を更新しない。インデックスは有効行向け部分インデックスに合わせて記載。 |

