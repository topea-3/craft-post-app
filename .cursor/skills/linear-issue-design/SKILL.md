---
name: linear-issue-design
description: Linear issue から docs/ に設計書を作成・更新するワークフロー。Issue のステータス管理、ブランチ確認・作成、既存 docs/ と実装コードの調査、自己レビュー、Linear ステータス更新まで行う。ユーザーが Linear issue の設計、設計書作成、docs 更新、issue 設計フローを依頼したときに使う。
disable-model-invocation: true
---

# Linear Issue 設計

Linear issue を起点に、プロジェクトの `docs/` に設計書を作成・更新する。

## ワークフロー

1. Linear issue を確認してステータスをチェックする。Issue IDが未指定の場合はユーザーに確認する。
2. issueがDoneならそのまま終了する。DoneまたはIn progress以外ならIn Progressにステータスを変更する。
3. 作業ブランチを確認し、必要なら作成またはチェックアウトする。
4. Issueの内容を確認し設計する内容を把握する。
5. @docs の内容と既存の実装コードを確認して、設計を行い@docs に設計書を作成、更新する。
6. 作成した設計書を自己レビューする。観点は以下の通り。
    - Issueに記載されていることが実現できているか。
    - 機能において矛盾がないか
    - 実装済みの機能と整合しているか
    - プロジェクトの方針、要件、アーキテクチャに整合しているか
    - 改善点はないか
7. 6のレビュー結果を基に5以降を実行する。問題点や改善点が解決したらIssueのステータスをレビューに変更する。
8. サマリを報告する

## ルール

- 設計に迷う部分は都度ユーザに質問する。その際は可能な限り選択肢を検討してから質問をする
- ブランチの checkout / 作成は Step 3。push / commit はユーザー明示依頼時のみ
- 新規ブランチ作成時はベースブランチをユーザーに確認してから作成する

## 実行手順

### Step 1–2: Linear issue とステータス

[共通リファレンス — Linear MCP / ステータス遷移](../linear-issue-shared/reference.md#linear-mcp) に従う。

- Issue ID 未指定 → ユーザーに確認
- `get_issue` で title / description / state / team / labels / relations を取得

### Step 3: ブランチ確認・作成

[共通リファレンス — ブランチ運用](../linear-issue-shared/reference.md#ブランチ運用) に従う。

### Step 4: 設計対象の把握

Issue から目的・受け入れ条件・対象機能/画面/API/DB・非スコープ・依存を整理する。不足は [質問テンプレート](../linear-issue-shared/reference.md#質問テンプレート) で確認。

### Step 5: 調査と設計書作成

1. 必読 docs・関連実装を調査（一覧は [reference.md](reference.md)）
2. 設計書を作成・更新（テンプレート・配置ルールは [reference.md](reference.md)）

### Step 6–7: 自己レビューと修正ループ

[reference.md — 自己レビュー](reference.md#自己レビュー) のチェックリストで記録する。指摘がある間は Step 5 に戻る。全項目クリア後、Issue を **Review** に更新。

### Step 8: サマリ報告

- Issue ID / タイトル / 最終ステータス
- 作業ブランチ名（新規作成時はベースブランチ名も）
- 作成・更新した設計書のパス
- 設計の要点（3–5 行）
- 自己レビュー結果
- 未決事項（あれば）

## 追加リソース

| 内容 | 参照先 |
|------|--------|
| Linear MCP・ブランチ・質問形式 | [../linear-issue-shared/reference.md](../linear-issue-shared/reference.md) |
| docs 構成・設計書テンプレート・レビュー詳細 | [reference.md](reference.md) |
