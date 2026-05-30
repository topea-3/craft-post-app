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
- 本番コード実装（Step 5）とユニットテスト（Step 6）は分けて行う。テストなしで Step 7 に進まない
- ブランチの checkout / 作成は Step 3 で行う。push はユーザー明示依頼時のみ
- **新規ブランチ作成時は、ベースブランチをユーザーに確認してから作成する**（自動で develop から切らない）

## 実行手順

### Step 1–2: Linear issue とステータス

- Linear MCP（`plugin-linear-linear`）を使う。ツール呼び出し前にスキーマを確認する。
- Issue ID 未指定 → ユーザーに ID（例: `CRA-123`）を確認する。
- `get_issue` で title / description / state / team / labels / relations / gitBranchName を取得する。
- ステータス判定:
  - **Done** → 実装不要。サマリのみ報告して終了。
  - **In Progress** → 変更しない。
  - **上記以外** → `save_issue` で `state: "In Progress"` に更新（正式名は `list_issue_statuses` で確認）。

### Step 3: ブランチ確認・作成

**命名規則**

| 種別 | 形式 | 例 |
|------|------|-----|
| 機能開発 | `feature/<issue id>_<description>` | `feature/CRA-123_sender-label-unique` |
| バグ修正 | `fix/<issue id>_<description>` | `fix/CRA-456_duplicate-label-race` |

- `<issue id>`: Linear の identifier（例: `CRA-123`）
- `<description>`: Issue タイトルから生成した短い slug（小文字英数字とハイフン、最大 40 文字程度）

**fix / feature の判定**

1. Issue labels に Bug / bug / fix 系があれば **fix**
2. タイトルが `Fix:` / `fix:` / `バグ修正` 等なら **fix**
3. 判断できない場合はユーザーに確認する

**手順**

```bash
git branch --show-current
git status --short
```

1. 現在ブランチが期待名と一致 → そのまま Step 4 へ
2. ローカルに期待ブランチが存在 → `git checkout <branch>`
3. リモートのみに存在 → `git checkout -t origin/<branch>`
4. **存在しない（新規作成が必要）** → 下記「ベースブランチの確認」を行ってから作成

**ベースブランチの確認（新規作成時は必須）**

ブランチを新規作成する前に、必ずユーザーにベースブランチを確認する。

```markdown
## 確認: 作業ブランチの作成

**作成するブランチ**: `feature/CRA-123_sender-label-unique`（例）

**ベースブランチを選んでください**
- A: `develop` から作成する（推奨。存在しない場合はその旨を伝える）
- B: 別のブランチを指定する（ブランチ名を入力してください）

**推奨**: 通常の機能開発・バグ修正は A（develop）
```

- **A を選択** → `develop` をベースにする（ローカルに無ければ `origin/develop` を fetch してから作成）
- **B を選択** → ユーザー指定のブランチ名をベースにする（存在確認してから checkout）
- `develop` が存在しない場合 → A 選択時はユーザーに `main` へフォールバックするか、B で指定してもらうか確認する

**作成コマンド（ベース確定後）**

```bash
git fetch origin
git checkout <base-branch>
git pull --ff-only origin <base-branch>   # 可能なら
git checkout -b feature/CRA-123_short-description
```

**注意**

- 未コミット変更があり checkout できない → ユーザーに stash / commit / 破棄の判断を確認
- `get_issue` の `gitBranchName` が命名規則に合致していればそれを優先してもよい
- 詳細は [reference.md](reference.md) のブランチ運用

### Step 4: 実装対象の把握

Issue と docs から以下を整理する。

- 受け入れ条件 / 完了条件
- 関連設計書（`docs/design/`、`docs/domain/`、`docs/mock-up/`）
- 変更対象レイヤー（DB / Rust / Tauri コマンド / React）
- 非スコープ
- 依存 issue・ブロッカー

設計書が無い、または Issue だけでは実装判断できない場合は、実装前にユーザーへ確認する（設計スキル `linear-issue-design` の実行を提案してもよい）。

### Step 5: 調査・計画・本番コード実装

**調査（実装前に必ず行う）**

1. 関連設計書・Issue 記述
2. 既存の類似実装（同 feature / 同レイヤーのファイル）
3. 触るファイルの import・型・エラーハンドリングの慣例
4. 既存テストの配置・命名・セットアップ（`setup_pool` 等）

**実装順序（下位レイヤーから。テストは Step 6）**

```
1. DB マイグレーション（必要な場合）  → src-tauri/migrations/
2. ドメイン                            → src-tauri/src/domain/
3. リポジトリ / インフラ               → src-tauri/src/infrastructure/
4. Tauri コマンド                      → src-tauri/src/lib.rs 等
5. フロントエンド                      → src/features/ / src/components/
```

**実装時の原則**

- 既存パターンをコピーして拡張する（新規抽象化は避ける）
- 1 レイヤーずつ実装し、都度 `cargo check` でコンパイル可能な状態を保つ
- エラーメッセージ・Validation 文言は既存の同種機能に合わせる

### Step 6: ユニットテスト（実装 → 実行 → 確認 → 修正）

本番コード完成後、テスト専用ステップとして以下を **全テスト成功まで繰り返す**。

```
テスト Progress:
- [ ] 5a. テストケースを洗い出す
- [ ] 5b. テストを実装する
- [ ] 5c. テストを実行する
- [ ] 5d. 結果を確認する
- [ ] 5e. 失敗があれば修正して 5c に戻る
```

**5a. テストケース洗い出し**

Issue の受け入れ条件と設計書の「テスト方針」「エッジケース」から、最低限以下をカバーする。

- 正常系（happy path）
- Validation エラー（入力不正・not found・archived 等）
- DB 制約・競合（UNIQUE 違反、部分インデックス等）
- 境界値（空配列、ページング境界、0 件）

**5b. テスト実装**

| 対象 | 配置 | 参照 |
|------|------|------|
| ドメインの不変条件 | 同一ファイル `#[cfg(test)] mod tests` | `domain/*/*.rs` |
| リポジトリ | `infrastructure/*/*_tests.rs` | `sqlx_sender_entry_repository_tests.rs` |
| Tauri コマンド | `command_tests.rs` | `create_sender_entry_impl` 等を直接呼ぶ |

- 既存テストの `setup_pool()` / `sample_*()` パターンを再利用する
- テスト名は `#[tokio::test] async fn <action>_<expected_outcome>()` 形式
- 意味のあるアサーションのみ（自明なテストは書かない）

**5c. テスト実行**

```bash
# 変更モジュールに絞る（推奨）
cd src-tauri && cargo test <module_or_test_name>

# 全体
cd src-tauri && cargo test
# または task test
```

**5d. 結果確認**

- 全テスト **passed** であること
- 新規追加テストが意図どおり実行されていること（filtered out されていないか）
- 警告のみで失敗扱いになっていないか

**5e. 修正**

| 失敗の種類 | 対応 |
|-----------|------|
| 本番コードのバグ | Step 5 に戻って修正 → Step 6 を再実行 |
| テストの期待値・セットアップ誤り | テストを修正 → 5c から再実行 |
| 既存テストの破壊（regression） | 本番コードの副作用を修正。意図的変更ならテストも更新 |

**Step 6 完了後の検証（Step 7 前に実行）**

```bash
npm run lint          # フロント ESLint
npm run build         # tsc + vite build
cd src-tauri && cargo test   # Rust テスト全体
```

Taskfile 利用時: `task lint` / `task check` / `task test` でも可。詳細は [reference.md](reference.md)。

### Step 7–8: 自己レビューと修正ループ

```
実装レビュー結果:
- [ ] Issue 受け入れ条件を満たす
- [ ] 設計書・docs と一致
- [ ] 既存パターン・レイヤー構成に沿う
- [ ] エッジケース・エラー処理が適切
- [ ] ユニットテストが要件・エッジケースをカバー
- [ ] lint / build / test 成功
- [ ] スコープ外変更・過剰実装なし
```

指摘がある間は Step 5 または Step 6 に戻って修正する。全項目クリア後、`save_issue` でステータスを **レビュー**（チームの Review 相当）に更新。

### Step 9: サマリ報告

- Issue ID / タイトル / 最終ステータス
- **作業ブランチ名**（新規作成した場合はその旨とベースブランチ名）
- 変更ファイル一覧（主要なもの）
- 実装内容の要点（3–5 行）
- **追加・更新したテスト**（ファイル名とテストケース概要）
- 実行した検証コマンドと結果
- 自己レビュー結果（問題なし / 修正した点）
- 未決事項・ユーザー確認事項（あれば）
- commit / PR は未実施であること（依頼がなければ）

## 質問の出し方

```markdown
## 確認: [論点]

**背景**: [なぜ判断が必要か]

**選択肢**
- A: [概要] — メリット / デメリット
- B: [概要] — メリット / デメリット

**推奨**: [理由付き]
```

## 追加リソース

- レイヤー構成・検証コマンド・レビュー詳細: [reference.md](reference.md)
