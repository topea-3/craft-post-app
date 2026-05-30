# Linear Issue ワークフロー — 共通リファレンス

`linear-issue-design` / `linear-issue-implement` スキル（`.cursor/skills/`）の Step 1–3 で参照する共通手順。

---

## Linear MCP

- サーバー: `plugin-linear-linear`（ツール呼び出し前にスキーマを確認）

| 操作 | ツール | 備考 |
|------|--------|------|
| Issue 取得 | `get_issue` | `id` に `CRA-123` 形式。`gitBranchName` も確認 |
| ステータス一覧 | `list_issue_statuses` | `team` 必須 |
| ステータス更新 | `save_issue` | `id` + `state` |

### ステータス遷移

| 現在 | 動作 |
|------|------|
| Done | 作業不要。サマリのみ報告して終了 |
| In Progress | 変更しない |
| 上記以外 | `In Progress` に更新 |
| 作業完了後 | チームの **Review** 相当に更新（`list_issue_statuses` で正式名を確認） |

`description` の Markdown はエスケープせずそのまま渡す。

---

## ブランチ運用

### 命名規則

| 種別 | 形式 | 例 |
|------|------|-----|
| 機能開発 | `feature/<issue id>_<description>` | `feature/CRA-123_sender-label-unique` |
| バグ修正 | `fix/<issue id>_<description>` | `fix/CRA-456_duplicate-label-race` |

### description slug

Issue タイトルから生成: 小文字化 → 非英数字をハイフン → 連続ハイフン圧縮 → 先後除去 → 約 40 文字で切り詰め。

例: `差出人ラベルの一意制約を追加` → `sender-label-unique`

### fix / feature 判定

| 条件 | プレフィックス |
|------|----------------|
| labels: Bug / bug / fix | `fix` |
| タイトル: Fix / fix / バグ修正 | `fix` |
| 上記以外 | `feature` |
| 判断不能 | ユーザーに確認 |

`get_issue` の `gitBranchName` が命名規則に合致すれば優先してもよい。

### チェックアウトフロー

```bash
git branch --show-current
git status --short
```

```
期待ブランチと一致?     → 続行
ローカルに存在?         → git checkout <branch>
リモートのみ?           → git checkout -t origin/<branch>
未作成?                 → ベースブランチをユーザーに確認 → git checkout -b
```

### ベースブランチ（新規作成時・ユーザー確認必須）

エージェントが独自判断でベースを決めて切らない。

```markdown
## 確認: 作業ブランチの作成

**作成するブランチ**: `feature/CRA-123_sender-label-unique`（例）

**ベースブランチを選んでください**
- A: `develop` から作成する（推奨。存在しない場合はその旨を伝える）
- B: 別のブランチを指定する（ブランチ名を入力してください）
```

| 選択 | ベース |
|------|--------|
| A | `develop`（無ければユーザーに再確認） |
| B | ユーザー指定（存在確認後） |

```bash
git fetch origin
git branch -a
git checkout <base-branch>
git pull --ff-only origin <base-branch>   # 失敗時はユーザーに確認
git checkout -b <expected-branch>
```

### 禁止事項

- `git push --force`（ユーザー明示依頼がない限り）
- 未確認の stash / hard reset
- 新規ブランチをユーザー確認なしで作成

詳細: [repository-and-branch-strategy.md](./repository-and-branch-strategy.md)

---

## 質問テンプレート

```markdown
## 確認: [論点]

**背景**: [なぜ判断が必要か]

**選択肢**
- A: [概要] — メリット / デメリット
- B: [概要] — メリット / デメリット

**推奨**: [理由付き]
```
