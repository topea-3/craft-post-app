# Linear Issue 設計 — リファレンス

`SKILL.md` がワークフローとルールを担う。本ファイルは Step 5（調査・設計書作成）と Step 6（自己レビュー）の参照資料のみ。

## 調査の優先順

### 必読 docs

1. `docs/overview/requirements-and-constraints.md`
2. `docs/overview/architecture-overview.md`
3. `docs/overview/decisions-summary.md`
4. 関連ドメイン: `docs/domain/`
5. 関連技術決定: `docs/tech-decisions/`
6. 関連モック: `docs/mock-up/`

### 実装調査（現状把握用）

- `src/features/`
- `src-tauri/src/`（domain / infrastructure / lib.rs）
- `src-tauri/migrations/`

## docs/ 構成

```
docs/
├── overview/
├── domain/
├── tech-decisions/
├── mock-up/
├── design/             # 機能横断・実装設計（新規作成の主な配置先）
└── project-setup/
```

## 設計書の配置

| 内容 | 配置先 |
|------|--------|
| 新規機能・横断設計 | `docs/design/<feature-name>-design.md` |
| ドメイン仕様の更新 | `docs/domain/` |
| 画面仕様 | `docs/mock-up/` または `docs/design/` |

## 設計書テンプレート

```markdown
# [機能名] — 設計書

- **Linear Issue**: [ID] [タイトル]
- **ステータス**: Draft / Reviewed
- **最終更新**: YYYY-MM-DD

---

## 1. 背景・目的

## 2. スコープ
### 対象
### 非スコープ

## 3. 要件
### 機能要件
### 非機能要件

## 4. 現状分析
### 関連 docs
### 関連実装
### ギャップ

## 5. 設計
### 5.1 データモデル / DB
### 5.2 API / Tauri コマンド
### 5.3 フロントエンド
### 5.4 フロー

## 6. エッジケース・エラー処理

## 7. テスト方針

## 8. 実装タスク（参考）

## 9. 未決事項
```

## 自己レビュー

Step 6–7 で使用する checklist。すべて `[x]` になるまで Step 5 に戻る。

```
- [ ] Issue 要件の充足
- [ ] 機能矛盾なし
- [ ] 実装済み機能との整合
- [ ] 方針・要件・アーキテクチャとの整合
- [ ] 改善点なし（または対応済み）
```

| 観点 | 確認内容 |
|------|----------|
| Issue 要件の充足 | 受け入れ条件が設計でカバーされているか。漏れ・過剰スコープがないか |
| 機能矛盾 | 同一機能内・関連機能間で仕様が矛盾していないか |
| 実装整合 | 既存 API / DB / UI と設計が食い違わないか |
| 方針・要件・アーキテクチャ | `requirements-and-constraints.md`、Tauri + SQLite、オフライン前提と整合するか |
| 改善点 | シンプル化、既存パターン再利用、将来拡張の余地 |

## プロジェクト前提（設計時）

- **スタック**: Tauri + React + TypeScript + SQLite
- **アーキテクチャ**: UI → Tauri コマンド → ドメイン → インフラ（sqlx）
- **永続化**: `src-tauri/migrations/`
- **オフライン**: 日常利用はオフライン完結

## 関連スキル

| スキル | 用途 |
|--------|------|
| `linear-issue-design` | 本スキル — docs/ に設計書を作成 |
| `linear-issue-implement` | 設計に基づきコードを実装 |

設計書が無い Issue で実装依頼が来た場合は、先に `linear-issue-design` の実行を提案する。
