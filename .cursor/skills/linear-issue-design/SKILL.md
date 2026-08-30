---
name: linear-issue-design
description: >-
  Linear issue から docs/ に設計書を作成・更新する。Issue ステータス管理、ブランチ確認、
  docs/ と実装コードの調査、自己レビューループ、Linear ステータス更新まで自律実行する。
  ユーザーが Linear issue の設計、設計書作成、docs 更新、issue 設計フローを依頼したときに使う。
disable-model-invocation: true
---

# Linear Issue 設計

Linear issue を起点に、プロジェクトの `docs/` に設計書を作成・更新する。

## 構成ファイルの責務

| ファイル | 責務 |
|----------|------|
| `SKILL.md`（本ファイル） | 起動条件、自律実行ワークフロー、不変ルール |
| `references/design-reference.md` | 調査先、docs 構成、設計書テンプレート、自己レビュー checklist |
| `docs/project-setup/linear-issue-workflow-common.md` | Linear MCP、ブランチ運用、質問テンプレート（design / implement 共通） |

## 自律実行ワークフロー

以下を上から順に実行する。Step 5 以降は **自己レビューがすべてクリアするまで** Step 5–7 を繰り返す。

```
Task Progress:
- [ ] Step 1–2: Linear issue 確認・ステータス更新
- [ ] Step 3: ブランチ確認・作成
- [ ] Step 4: 設計対象の把握
- [ ] Step 5: 調査と設計書作成
- [ ] Step 6–7: 自己レビューと修正ループ
- [ ] Step 8: サマリ報告
```

### Step 1–2: Linear issue とステータス

[Linear MCP](../../../docs/project-setup/linear-issue-workflow-common.md#linear-mcp) に従う。

- Issue ID 未指定 → ユーザーに確認
- Done → 作業不要で Step 8（終了サマリのみ）
- In Progress → 変更しない
- 上記以外 → In Progress に更新
- `get_issue` で title / description / state / team / labels / relations を取得

### Step 3: ブランチ確認・作成

[ブランチ運用](../../../docs/project-setup/linear-issue-workflow-common.md#ブランチ運用) に従う。

### Step 4: 設計対象の把握

Issue から目的・受け入れ条件・対象（機能 / 画面 / API / DB）・非スコープ・依存を整理する。不足は [質問テンプレート](../../../docs/project-setup/linear-issue-workflow-common.md#質問テンプレート) で確認。

### Step 5: 調査と設計書作成

1. [references/design-reference.md](references/design-reference.md) を読む
2. 必読 docs・関連実装を調査
3. 設計書テンプレートに従い `docs/` に作成・更新

### Step 6–7: 自己レビューと修正ループ

1. [自己レビュー checklist](references/design-reference.md#自己レビュー) の全項目を確認し、結果を記録する
2. 未達項目がある → 設計書を修正（Step 5）→ Step 6 から再実行
3. **すべてクリア** → Issue を Review に更新 → Step 8 へ

### Step 8: サマリ報告

- Issue ID / タイトル / 最終ステータス
- 作業ブランチ名（新規作成時はベースブランチ名も）
- 作成・更新した設計書のパス
- 設計の要点（3–5 行）
- 自己レビュー結果
- 未決事項（あれば）

## 不変ルール

- 設計に迷う部分は都度ユーザーに質問する。可能な限り選択肢を提示してから質問する
- ブランチの checkout / 作成は Step 3 のみ。commit / push はユーザー明示依頼時のみ
- 新規ブランチ作成時はベースブランチをユーザーに確認してから作成する
