# Linear Issue ワークフロー — 共通リファレンス

`linear-issue-design` / `linear-issue-implement` スキル（`.cursor/skills/`）の Step 1–3 および作業完了時のステータス更新で参照する共通手順。

---

## Linear MCP

- サーバー: `plugin-linear-linear`（ツール呼び出し前にスキーマを確認）

| 操作 | ツール | 備考 |
|------|--------|------|
| Issue 取得 | `get_issue` | `id` に `CRA-123` 形式。`gitBranchName` も確認 |
| ステータス一覧 | `list_issue_statuses` | `team` 必須 |
| ステータス更新 | `save_issue` | `id` + `state`（正式名を使用） |

`description` の Markdown はエスケープせずそのまま渡す。

### Step 1–2: Issue 取得とステータス更新

**必ず以下の順序で実行する**（状態未取得のまま更新しない）。

1. Issue ID 未指定 → ユーザーに確認
2. `get_issue` で title / description / state / team / labels / relations / gitBranchName を取得
3. `list_issue_statuses`（`team` 必須）でチームの正式ステータス名を確認
4. 現在ステータスに応じて以下を実行:

**判定ルール**: `name`（正式名）を先に見る。`type` は補助（`completed` / `canceled` → 終端、`unstarted` / `backlog` → 着手前）。In Progress と Review はどちらも `type=started` になりうるため、**`type` だけで Review 中を判定しない**。

| 分類 | 判定 | 動作 |
|------|------|------|
| 終端 | Done / Canceled / Duplicate 相当 | 作業不要。サマリのみ報告して終了 |
| ブロック中 | Blocked / On Hold / Parked 相当 | In Progress にしない。ユーザーに継続可否を確認 |
| レビュー中 | 名前が Review / In Review / レビュー | **In Progress に戻さない**。ユーザーに継続可否を確認（下記） |
| 作業中 | In Progress 相当 | 変更しない |
| 着手前 | 上記以外（Todo / Backlog 等） | In Progress 相当の正式名で `save_issue` 更新 |

名称が不明な場合は候補一覧をユーザーに確認する。

#### レビュー中 / ブロック中の継続確認

- **続行する** → ステータスはそのまま（In Progress に戻さない）。Step 3 以降を実行
- **続行しない** → サマリのみ報告して終了
- **判断不能** → 候補一覧を出してユーザーに確認

### 作業完了時のステータス更新

1. 現在が Review 相当 → 更新不要（既に Review）。Step 8 / 9 へ
2. 上記以外 → `list_issue_statuses` で Review 相当の正式名を確認し `save_issue` で更新
3. **失敗時** — 再試行せず、取得した候補名をユーザーに確認する

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
未コミット変更で checkout 不可 → ユーザーに stash / commit / 破棄を確認
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
