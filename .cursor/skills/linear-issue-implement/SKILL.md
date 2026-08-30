---
name: linear-issue-implement
description: >-
  Linear issue と docs/ の設計に基づき実装する。Issue ステータス管理、ブランチ確認、
  設計書・既存コードの調査、段階的実装、ユニットテスト、lint/test 検証、
  自己レビューループ、Linear ステータス更新まで自律実行する。
  ユーザーが Linear issue の実装、機能追加、バグ修正、issue 実装フローを依頼したときに使う。
disable-model-invocation: true
---

# Linear Issue 実装

Linear issue を起点に、設計書・既存実装に沿ってコードを実装する。

## 構成ファイルの責務

| ファイル | 責務 |
|----------|------|
| `SKILL.md`（本ファイル） | 起動条件、自律実行ワークフロー、不変ルール |
| `references/implement-reference.md` | レイヤー慣例、テスト手順、検証コマンド、実装パターン、自己レビュー checklist |
| `docs/project-setup/linear-issue-workflow-common.md` | Linear MCP、ステータス遷移、ブランチ運用、質問テンプレート（design / implement 共通） |

## 自律実行ワークフロー

以下を上から順に実行する。Task Progress は作業メモに複製し、進むごとに更新する。

Step 5–8 は **ブロッカー項目がすべてクリアするまで** 繰り返す（最大 3 周）。収束しない場合は未達項目と選択肢をユーザーに確認して停止する。Step 9（サマリ）はループに含めない。

```
Task Progress:
- [ ] Step 1–2: Linear issue 確認・ステータス更新
- [ ] Step 3: ブランチ確認・作成
- [ ] Step 4: 実装対象の把握
- [ ] Step 5: 本番コード実装
- [ ] Step 6: ユニットテスト
- [ ] Step 7–8: 自己レビューと修正ループ
- [ ] Step 9: サマリ報告
```

### Step 1–2: Linear issue とステータス

[Step 1–2: Issue 取得とステータス更新](../../../docs/project-setup/linear-issue-workflow-common.md#step-12-issue-取得とステータス更新) に従う。

### Step 3: ブランチ確認・作成

[ブランチ運用](../../../docs/project-setup/linear-issue-workflow-common.md#ブランチ運用) に従う。

### Step 4: 実装対象の把握

Issue・設計書から受け入れ条件・変更レイヤー・非スコープ・依存を整理する。設計書が無い場合は `linear-issue-design` の実行を提案する。

### Step 5: 本番コード実装

1. [references/implement-reference.md](references/implement-reference.md) を読む
2. 関連設計書・類似実装・既存テスト慣例を調査
3. 下位レイヤーから順に実装: migration → domain → infrastructure → Tauri コマンド → frontend
4. レイヤーごとに `cargo check` でコンパイル可能な状態を保つ

### Step 6: ユニットテスト

[ユニットテスト](references/implement-reference.md#ユニットテスト) の手順（洗い出し → 実装 → 実行 → 確認 → 修正）に従う。6c–6e は最大 3 回。環境・ツールチェーン・既存テストの先行失敗で通らない場合は、原因と選択肢をユーザーに確認して停止する（Step 5 に戻って無関係な修正をしない）。

完了後、[検証コマンド](references/implement-reference.md#検証コマンド) の「一式（上記を全て）」を実行する。

### Step 7–8: 自己レビューと修正ループ

1. [自己レビュー checklist](references/implement-reference.md#自己レビュー) の **ブロッカー** 項目を確認し、結果を記録する
2. ブロッカー未達 → 原因に応じ Step 5 または Step 6 に戻り修正 → Step 7 から再実行（最大 3 周）
3. 3 周以内にブロッカーすべてクリア → [作業完了時のステータス更新](../../../docs/project-setup/linear-issue-workflow-common.md#作業完了時のステータス更新) に従い Issue を更新 → Step 9 へ
4. 3 周超過 → 未達項目と選択肢をユーザーに確認して停止

### Step 9: サマリ報告

- Issue ID / タイトル / 最終ステータス
- 作業ブランチ名（新規作成時はベースブランチ名も）
- 変更ファイル一覧（主要なもの）
- 実装要点（3–5 行）
- 追加・更新したテスト
- 検証コマンドと結果
- 自己レビュー結果
- 未決事項（あれば）
- commit / PR 未実施（依頼がなければ）

## 不変ルール

- 実装に迷う部分は都度ユーザーに質問する。可能な限り選択肢を提示してから質問する
- commit / push はユーザー明示依頼時のみ
- 設計書にない変更が必要な場合は、実装前にユーザーに確認する
- スコープを最小限に保ち、Issue と設計書の範囲外は変更しない
- 本番コード（Step 5）とユニットテスト（Step 6）は分ける。テストなしで Step 7 に進まない
- ブランチの checkout / 作成は Step 3 のみ
- 新規ブランチ作成時はベースブランチをユーザーに確認してから作成する
