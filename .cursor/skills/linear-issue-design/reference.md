# Linear Issue 設計 — リファレンス

## docs/ 構成

```
docs/
├── overview/           # 要件・アーキテクチャ・決定事項サマリ
├── domain/             # ドメイン仕様（address, sender 等）
├── tech-decisions/     # 技術選定の記録
├── mock-up/            # 画面モック・UI 仕様
├── design/             # 機能横断・実装設計（本スキルで新規作成する場所）
└── project-setup/      # 開発環境・運用
```

## 設計書テンプレート

新規設計書は `docs/design/<feature-name>-design.md` に作成する。

```markdown
# [機能名] — 設計書

- **Linear Issue**: [ID] [タイトル]
- **ステータス**: Draft / Reviewed
- **最終更新**: YYYY-MM-DD

---

## 1. 背景・目的

[Issue から。なぜ必要か]

## 2. スコープ

### 対象

- ...

### 非スコープ

- ...

## 3. 要件

### 機能要件

- ...

### 非機能要件

- ...

## 4. 現状分析

### 関連 docs

- `docs/...`

### 関連実装

- `src/...` / `src-tauri/...`

### ギャップ

- 現状と目標の差分

## 5. 設計

### 5.1 データモデル / DB

[テーブル・カラム・制約。マイグレーション方針]

### 5.2 API / Tauri コマンド

[コマンド名・入出力・エラー]

### 5.3 フロントエンド

[画面・コンポーネント・状態管理]

### 5.4 フロー

[主要ユースケース。必要なら mermaid]

## 6. エッジケース・エラー処理

- ...

## 7. テスト方針

- ...

## 8. 実装タスク（参考）

- [ ] ...

## 9. 未決事項

- ...
```

## Linear MCP 操作

| 操作 | ツール | 備考 |
|------|--------|------|
| Issue 取得 | `get_issue` | `id` に `CRA-123` 形式 |
| ステータス一覧 | `list_issue_statuses` | `team` 必須。Review / In Progress の正式名を確認 |
| ステータス更新 | `save_issue` | `id` + `state`（名前または ID） |

Markdown の `description` はエスケープせず、そのまま渡す（Linear MCP の指示に従う）。

## 自己レビュー観点の詳細

| 観点 | 確認内容 |
|------|----------|
| Issue 要件の充足 | 受け入れ条件が設計でカバーされているか。漏れ・過剰スコープがないか |
| 機能矛盾 | 同一機能内・関連機能間で仕様が矛盾していないか |
| 実装整合 | 既存 API / DB / UI の挙動と設計が食い違わないか。破壊的変更の有無 |
| 方針・要件・アーキテクチャ | `requirements-and-constraints.md`、Tauri + SQLite 構成、オフライン前提等と整合するか |
| 改善点 | シンプル化、既存パターン再利用、将来拡張の余地 |

## プロジェクト固有の前提

- **スタック**: Tauri + React + TypeScript + SQLite（Rust バックエンド）
- **アーキテクチャ**: UI → Tauri コマンド → ドメイン → インフラ（sqlx）
- **永続化**: `src-tauri/migrations/` でスキーマ管理
- **オフライン**: 日常利用はオフライン完結（`requirements-and-constraints.md`）
