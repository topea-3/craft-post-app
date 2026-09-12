## PostcardReceipt 編集画面モック（v1）

- **画面ID**: `REC003`
- **画面名**: 受取履歴編集
- **設計書**: `docs/design/postcard-receipt-v1-design.md`

---

### 1. 画面レイアウト

- **上部エリア**
  - 左上: 画面タイトル「受取履歴編集」
  - 右上: 「キャンセル」リンク（詳細または一覧へ）

- **フォーム**: 作成画面（`REC002`）と同一項目
  - 初期値: `get_postcard_receipt` の結果を反映
  - 紐付け方法の切替: 表示名のみ ↔ 住所録選択の相互変換可

---

### 2. ボタン・アクション

- **キャンセル**: 編集中なら確認ダイアログ → 詳細（`REC004`）
- **保存**: `update_postcard_receipt` → 詳細（`REC004`）+ フラッシュ「受取履歴を更新しました。」

---

### 3. 追加仕様

- 匿名受取 → 後から住所録紐付け: `addressEntryId` を設定、`senderDisplayName` は空にしても可
- 紐付け済み → 表示名のみへ変更: `addressEntryId` を null にし `senderDisplayName` 必須

---

### 4. UX メモ

- 作成画面と共通 `PostcardReceiptForm` コンポーネントを想定
