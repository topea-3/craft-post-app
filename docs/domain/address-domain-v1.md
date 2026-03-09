# Craft Post App — 宛名住所情報ドメイン（v1）

## 1. ドメインオブジェクト全体像

- **エンティティ**
  - `AddressEntry`（住所録エントリ）
- **値オブジェクト**
  - `PersonName`（氏名）
  - `Honorific`（敬称）
  - `PostalCode`（郵便番号）
  - `Address`（住所本体）
  - `Memo`（メモ／備考）

> v1 では「1 件の住所に対して 1 件の住所録エントリ」を基本としつつ、  
> **連名（複数人の氏名）** を `AddressEntry` 内で表現できるようにする。

将来機能（送受信履歴 `PostcardHistory`、グループ `ContactGroup` など）は v1 のスコープ外としつつ、  
`AddressEntry` を中心に拡張しやすい形を意識して設計します。

---

## 2. エンティティ

### 2.1 AddressEntry（住所録エントリ）

**役割**: はがきの宛先（および将来の差出人）として利用される 1 人〜複数人分の住所情報を表す。  
**識別子**: 永続化層の主キー `id` によって一意に識別される。

- **属性構成（論理モデル）**
  - `id` : 内部 ID（例: UUID / 数値 ID）
  - `primaryName` : 主たる氏名を表す `PersonName` 値オブジェクト
  - `coRecipients` : 連名用の `PersonName` 値オブジェクトの配列（0 件以上）
  - `honorific` : `Honorific` 値オブジェクト（連名全体に対して 1 つ）
  - `postalCode` : `PostalCode` 値オブジェクト
  - `address` : `Address` 値オブジェクト
  - `memo` : `Memo` 値オブジェクト（任意）
  - `createdAt` : 作成日時
  - `updatedAt` : 更新日時
  - `archived` : 論理削除フラグ（将来の履歴参照に備える）
- **主なドメインルール**
  - `primaryName` と `address` は必須（空の住所録エントリは作らない）。
  - 連名は `coRecipients` として 0 件以上保持できる（例: 夫婦連名、家族連名）。
  - 宛名印字時の表示名は、`primaryName` と `coRecipients` を結合した上で `honorific` を末尾に付与する
    - 例: `["山田 太郎", "山田 花子"] + "様"` → `"山田 太郎・山田 花子 様"`
  - `archived = true` のエントリは通常の一覧からは非表示とする（※「アーカイブ一覧」は将来検討）。
  - 将来、送受信履歴 `PostcardHistory` 側から `AddressEntry` を参照する前提で、  
  物理削除よりも **論理削除（archive）を基本方針** とする。

---

## 3. 値オブジェクト

### 3.1 PersonName（氏名）
**役割**: 人名表記を表し、画面表示・ソート・敬称付与の基礎となる。  
**利用箇所**: `AddressEntry.primaryName`（主名）および `AddressEntry.coRecipients`（連名者）で再利用する。
- **属性**
  - `full` : 表示用の氏名（例: `"山田 太郎"`）
  - `last` : 姓（例: `"山田"`）
  - `first` : 名（例: `"太郎"`）
  - `kanaLast` : 姓（カナ）
  - `kanaFirst` : 名（カナ）
- **不変条件（例）**
  - `full` は非空。
  - `last` / `first` も v1 から必須（空文字は禁止）とする。
  - 文字数上限（例: 64〜128 文字）を超えない。
- **補助的な派生値**
  - `displayWithHonorific(honorific)` : `Honorific` と組み合わせた表示名（例: `"山田 太郎 様"`）。
  - 連名表示のためのユーティリティ（例）:
    - `joinRecipients(primaryName, coRecipients, honorific)`  
      → `["山田 太郎", "山田 花子"] + "様"` を `"山田 太郎・花子 様"` に整形する。（姓が一致している場合は連名の姓は省略する）

---

### 3.2 Honorific（敬称）

**役割**: 宛名の末尾に付与される敬称を表す。

- **プリセット候補（v1 で採用するセット）**
  - `"様"`
  - `"御中"`
  - `"ご家族様"`
  - `"なし"`
- **方針**
  - v1 では上記プリセットからの選択形式とし、カスタム敬称はサポートしない。
- **不変条件**
  - 上記プリセット値以外は受け付けない（UI 側でも選択のみに制限）。

---

### 3.3 PostalCode（郵便番号）

**役割**: 日本の郵便番号（7 桁）を表す。

- **内部表現案**
  - `value` : 数字 7 桁（`"1234567"`）で保持
  - `formatted` : 表示用にハイフンを含めた `"123-4567"` 形式を返すメソッドを用意
- **不変条件**
  - 0〜9 の数字のみ。
  - 桁数は必ず 7 桁。

---

### 3.4 Address（住所本体）

**役割**: 日本語住所を構造化して表現する。

- **属性（日本住所向け / v1 確定）**
  - `prefecture` : 都道府県（例: `"東京都"`）※必須
  - `city` : 市区町村（例: `"渋谷区"`）※必須
  - `street` : 町名・番地（例: `"神南 1-1-1"`）※必須
  - `building` : 建物名・部屋番号（例: `"○○ビル 3F"`）※任意
- **UI／DB との対応**
  - 入力も `prefecture` / `city` / `street` / `building` の 4 項目をそれぞれ別フィールドとして扱う。
  - `prefecture` はプリセットからの選択形式とする（自由入力は不可）。
  - DB レベルでも `prefecture` / `city` / `street` / `building` の 4 カラムで管理する。
- **不変条件**
  - `prefecture` / `city` / `street` は非空。
  - 文字数上限を設ける（例: 各 128〜256 文字）。
- **派生値**
  - `toSingleLine()` : `"東京都渋谷区神南 1-1-1 ○○ビル 3F"` のような 1 行表現。

---

### 3.5 Memo（メモ／備考）

**役割**: 住所録エントリに紐づく自由記述メモ。

- **属性**
  - `text` : 本文
- **制約**
  - 最大文字数（例: 1000 文字程度）を設け、無制限な長文を防ぐ。
  - 検索対象に含めるか（全文検索対象にするか）は実装時に判断。

---

## 4. 一覧・詳細画面とドメインの対応

### 4.1 一覧画面（AddressEntry List View）

- **表示対象**
  - `archived = false` の `AddressEntry` を対象とする。
- **想定表示項目**
  - 氏名＋敬称:
    - 単名: `primaryName.displayWithHonorific(honorific)`
    - 連名: `joinRecipients(primaryName, coRecipients, honorific)`
  - 郵便番号: `PostalCode.formatted`
  - 住所: `Address.toSingleLine()`
  - メモ（先頭数十文字をプレビュー）
  - 最終更新日時: `updatedAt`
- **ソート／フィルタ・検索（v1 のたたき台）**
  - デフォルトソート: `updatedAt` 降順、または 氏名昇順。
  - フリーテキスト検索:
    - 対象: `PersonName.full`, `Address.toSingleLine()`, `Memo.text`

---

### 4.2 詳細／編集画面（AddressEntry Detail/Edit View）

- **入力項目と対応ドメイン**
  - 主たる氏名: `primaryName.full`（必要に応じて姓・名を分割入力）
  - 連名: `coRecipients` として複数の `PersonName.full` を入力できるようにする（行追加式 UI など）
  - 敬称: `Honorific`
  - 郵便番号: `PostalCode`
  - 住所 1: `Address.city + Address.street`（UI 上の 1 フィールド）
  - 住所 2: `Address.building`
  - 都道府県: `Address.prefecture`
  - メモ: `Memo`
- **入力バリデーション（ドメイン側の考え方）**
  - 必須: 氏名、郵便番号、都道府県、市区町村、町名・番地。
  - 文字数制限: 氏名・住所・メモに上限を設ける。
  - 郵便番号形式: `PostalCode` の不変条件に準拠。
  - 敬称: 定義済みの候補のみ選択可能（v1）。

---

## 5. ライフサイクルとユースケース（Create/Update/Delete）

### 5.1 作成（CreateAddressEntry）

- 入力フォームから値を受け取り、各値オブジェクトの不変条件に従って `AddressEntry` を生成。
- 永続化成功後、一覧へリダイレクトまたは詳細画面へ遷移。

### 5.2 更新（UpdateAddressEntry）

- 既存の `AddressEntry` をロードし、編集フォームに展開。
- 変更後の入力で再度値オブジェクトを構築・検証し、問題なければ保存。

### 5.3 削除（ArchiveAddressEntry / DeleteAddressEntry）

- v1 の基本方針として **論理削除（アーカイブ）** を採用:
  - `archived = true` に更新する `ArchiveAddressEntry` ユースケースを主とする。
  - 物理削除は「明らかに誤って作ったダミーデータ」など、限定的なケースに絞る想定。
- 将来、送受信履歴と紐付いた場合も `AddressEntry` 自体は残し、履歴から参照可能にする。

---

## 6. 今後の検討ポイント（メモ）

- CSV インポート／エクスポートの仕様（カラム構成・文字コードなど）は、機能追加が必要になったタイミングで別途検討する。
- 将来の `PostcardHistory` やグループ機能との関連は、実際に機能を導入する段階でスキーマに追加する。

---

## 変更履歴


| 日付         | 内容                                            |
| ---------- | --------------------------------------------- |
| 2026-03-09 | 初版。TOP-15 のたたき台として、住所録ドメインをエンティティ・値オブジェクトで整理。 |


