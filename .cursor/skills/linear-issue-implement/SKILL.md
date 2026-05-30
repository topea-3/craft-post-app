---
name: linear-issue-implement
description: Linear issue と docs/ の設計に基づき実装を行うワークフロー。Issue ステータス管理、ブランチ確認・作成、設計書・既存コードの調査、段階的実装、ユニットテストの実装・実行・修正、lint/test 検証、自己レビュー、Linear ステータス更新まで行う。ユーザーが Linear issue の実装、機能追加、バグ修正、issue 実装フローを依頼したときに使う。
disable-model-invocation: true
---

# Linear Issue 実装

Linear issue を起点に、設計書・既存実装に沿ってコードを実装する。

## ワークフロー

1. Linear issue を確認してステータスをチェックする。Issue IDが未指定の場合はユーザーに確認する。
2. issueがDoneならそのまま終了する。DoneまたはIn progress以外ならIn Progressにステータスを変更する。
3. 作業ブランチを確認し、必要なら作成またはチェックアウトする。
4. Issueの内容と関連する設計書・docs を確認し、実装する内容を把握する。
5. 既存の実装コードを調査し、実装計画を立ててから段階的に本番コードを実装する。
6. ユニットテストを実装し、実行して結果を確認する。失敗した場合はテストまたは本番コードを修正し、全テストが通るまで繰り返す。
7. 実装結果を自己レビューする。観点は以下の通り。
    - Issue の受け入れ条件を満たしているか
    - 設計書・docs と実装が一致しているか
    - 既存コードのパターン・命名・レイヤー構成に沿っているか
    - エッジケース・エラー処理が適切か
    - ユニットテストが要件・エッジケースをカバーしているか
    - lint / 型チェック / テストが通るか
    - スコープ外の変更や過剰実装がないか
8. 7のレビュー結果を基に5以降を実行する。問題点や改善点が解決したらIssueのステータスをレビューに変更する。
9. サマリを報告する

## ルール

- 実装に迷う部分は都度ユーザに質問する。その際は可能な限り選択肢を検討してから質問をする
- ユーザーが明示的に依頼しない限り git commit / push は行わない
- 設計書にない変更が必要な場合は、実装前にユーザーに確認する
- スコープを最小限に保ち、Issue と設計書の範囲外は変更しない
- 本番コード（Step 5）とユニットテスト（Step 6）は分けて行う。テストなしで Step 7 に進まない
- ブランチの checkout / 作成は Step 3。push はユーザー明示依頼時のみ
- 新規ブランチ作成時はベースブランチをユーザーに確認してから作成する

## 実行手順

### Step 1–2: Linear issue とステータス

[共通リファレンス — Linear MCP / ステータス遷移](../linear-issue-shared/reference.md#linear-mcp) に従う。

- Issue ID 未指定 → ユーザーに確認
- `get_issue` で title / description / state / team / labels / relations / gitBranchName を取得

### Step 3: ブランチ確認・作成

[共通リファレンス — ブランチ運用](../linear-issue-shared/reference.md#ブランチ運用) に従う。

- 未コミット変更で checkout 不可 → ユーザーに stash / commit / 破棄を確認

### Step 4: 実装対象の把握

Issue・設計書から受け入れ条件・変更レイヤー・非スコープ・依存を整理する。設計書が無い場合は `linear-issue-design` の実行を提案する。

### Step 5: 本番コード実装

1. 関連設計書・類似実装・既存テスト慣例を調査（[reference.md](reference.md)）
2. 下位レイヤーから順に実装: migration → domain → infrastructure → Tauri コマンド → frontend
3. レイヤーごとに `cargo check` でコンパイル可能な状態を保つ

### Step 6: ユニットテスト

[reference.md — ユニットテスト](reference.md#ユニットテスト) の手順（洗い出し → 実装 → 実行 → 確認 → 修正）に従い、**全テスト成功まで繰り返す**。

完了後、lint / build / テスト全体を実行（[reference.md — 検証コマンド](reference.md#検証コマンド)）。

### Step 7–8: 自己レビューと修正ループ

[reference.md — 自己レビュー](reference.md#自己レビュー) のチェックリストで記録する。指摘がある間は Step 5 または Step 6 に戻る。全項目クリア後、Issue を **Review** に更新。

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

## 追加リソース

| 内容 | 参照先 |
|------|--------|
| Linear MCP・ブランチ・質問形式 | [../linear-issue-shared/reference.md](../linear-issue-shared/reference.md) |
| レイヤー慣例・テスト・検証・実装パターン・レビュー詳細 | [reference.md](reference.md) |
