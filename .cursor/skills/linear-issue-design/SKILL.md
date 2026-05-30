---
name: linear-issue-design
description: Linear issue から docs/ に設計書を作成・更新するワークフロー。Issue のステータス管理、既存 docs/ と実装コードの調査、自己レビュー、Linear ステータス更新まで行う。ユーザーが Linear issue の設計、設計書作成、docs 更新、issue 設計フローを依頼したときに使う。
---

# Linear Issue 設計

Linear issue を起点に、プロジェクトの `docs/` に設計書を作成・更新する。

## ワークフロー

1. Linear issue を確認してステータスをチェックする。Issue IDが未指定の場合はユーザーに確認する。
2. issueがDoneならそのまま終了する。DoneまたはIn progress以外ならIn Progressにステータスを変更する。
3. Issueの内容を確認し設計する内容を把握する。
4. @docs の内容と既存の実装コードを確認して、設計を行い@docs に設計書を作成、更新する。
5. 作成した設計書を自己レビューする。観点は以下の通り。
    - Issueに記載されていることが実現できているか。
    - 機能において矛盾がないか
    - 実装済みの機能と整合しているか
    - プロジェクトの方針、要件、アーキテクチャに整合しているか
    - 改善点はないか
6. 5のレビュー結果を基に4以降を実行する。問題点や改善点が解決したらIssueのステータスをレビューに変更する。
7. サマリを報告する

## ルール

- 設計に迷う部分は都度ユーザに質問する。その際は可能な限り選択肢を検討してから質問をする

## 実行手順

### Step 1–2: Linear issue とステータス

- Linear MCP を使う（`plugin-linear-linear`）。ツール呼び出し前にスキーマを確認する。
- Issue ID 未指定 → ユーザーに ID（例: `CRA-123`）を確認してから進める。
- `get_issue` で title / description / state / team / labels / relations を取得する。
- ステータス判定:
  - **Done** → 設計不要。サマリのみ報告して終了。
  - **In Progress** → 変更しない。
  - **上記以外** → `save_issue` で `state: "In Progress"` に更新（チーム固有名の場合は `list_issue_statuses` で確認）。

### Step 3: 設計対象の把握

Issue から以下を整理し、設計書のスコープを明確にする。

- 目的・背景
- 受け入れ条件 / 完了条件
- 対象機能・画面・API・DB
- 非スコープ（やらないこと）
- 関連 issue / 依存

不足があれば、選択肢付きでユーザーに確認する。

### Step 4: 調査と設計書作成

**必読 docs（優先順）**

1. `docs/overview/requirements-and-constraints.md`
2. `docs/overview/architecture-overview.md`
3. `docs/overview/decisions-summary.md`
4. 関連ドメイン: `docs/domain/`
5. 関連技術決定: `docs/tech-decisions/`
6. 関連モック: `docs/mock-up/`

**実装調査**

- 関連 feature: `src/features/`
- Tauri コマンド / ドメイン / インフラ: `src-tauri/src/`
- マイグレーション: `src-tauri/migrations/`

**設計書の配置**

- 新規機能・横断設計 → `docs/design/`（なければ作成）
- ドメイン仕様の更新 → `docs/domain/`
- 画面仕様 → `docs/mock-up/` または `docs/design/`

テンプレートと配置ルールは [reference.md](reference.md) を参照。

### Step 5–6: 自己レビューと修正ループ

レビュー結果はチェックリスト形式で記録する。

```
レビュー結果:
- [ ] Issue 要件の充足
- [ ] 機能矛盾なし
- [ ] 実装済み機能との整合
- [ ] 方針・要件・アーキテクチャとの整合
- [ ] 改善点なし（または対応済み）
```

指摘がある間は Step 4 に戻り設計書を修正する。全項目クリア後、`save_issue` でステータスを **レビュー**（チームの Review 相当）に更新。ステータス名は `list_issue_statuses` で確認する。

### Step 7: サマリ報告

以下を含めて報告する。

- Issue ID / タイトル / 最終ステータス
- 作成・更新した設計書のパス
- 設計の要点（3–5 行）
- 自己レビュー結果（問題なし / 修正した点）
- 未決事項・ユーザー確認事項（あれば）

## 質問の出し方

設計に迷うときは、次の形式でユーザーに確認する。

```markdown
## 確認: [論点]

**背景**: [なぜ判断が必要か]

**選択肢**
- A: [概要] — メリット / デメリット
- B: [概要] — メリット / デメリット
- C: [概要] — メリット / デメリット（必要なら）

**推奨**: [理由付き。なければ「未定」]
```

## 追加リソース

- 設計書テンプレート・docs 構成: [reference.md](reference.md)
