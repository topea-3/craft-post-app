# Linear Issue 実装 — リファレンス

## ブランチ運用

### 命名規則

| 種別 | 形式 | 例 |
|------|------|-----|
| 機能開発 | `feature/<issue id>_<description>` | `feature/CRA-123_sender-label-unique` |
| バグ修正 | `fix/<issue id>_<description>` | `fix/CRA-456_duplicate-label-race` |

### description slug の生成

Issue タイトルから以下のルールで生成する。

1. 小文字化
2. 英数字以外はハイフンに置換
3. 連続ハイフンを 1 つに
4. 先頭末尾のハイフンを除去
5. 40 文字程度で切り詰め

例: `差出人ラベルの一意制約を追加` → `sender-label-unique`

### fix / feature 判定

| 条件 | プレフィックス |
|------|----------------|
| labels: Bug / bug / fix | `fix` |
| タイトル: Fix / fix / バグ修正 | `fix` |
| 上記以外 | `feature` |
| 判断不能 | ユーザーに確認 |

### ベースブランチ（新規作成時）

**原則: ユーザー確認必須。** エージェントが独自判断でベースを決めてブランチを切らない。

| ユーザーの選択 | ベース |
|----------------|--------|
| A: develop から | `develop`（無ければユーザーに再確認） |
| B: 別ブランチ指定 | ユーザー入力のブランチ名 |

作成前の共通手順:

```bash
git fetch origin
git branch -a                    # ベースブランチの存在確認
git checkout <base-branch>
git pull --ff-only origin <base-branch>   # 失敗時はユーザーに確認
git checkout -b <expected-branch>
```

### チェックアウトフロー

```
現在ブランチ == 期待ブランチ?  → YES: 続行
ローカルに期待ブランチあり?    → YES: git checkout
リモートのみに存在?            → git checkout -t origin/<branch>
どちらもなし?                  → ユーザーにベースブランチを確認 → git checkout -b
```

### 禁止事項

- `git push --force`（ユーザー明示依頼がない限り）
- 未確認の stash / hard reset
- main への直接 commit（ブランチ運用方針に従う）

参照: `docs/project-setup/repository-and-branch-strategy.md`

## プロジェクト構成

```
src-tauri/
├── migrations/              # SQLite スキーマ（連番 SQL）
└── src/
    ├── domain/              # エンティティ・値オブジェクト・リポジトリ trait
    ├── infrastructure/      # sqlx 実装
    ├── lib.rs               # Tauri コマンド・DTO
    └── command_tests.rs     # コマンド統合テスト

src/
├── features/                # 画面・hooks・types（address, sender 等）
└── components/              # 共通 UI（form, PaginationControls 等）
```

## 実装レイヤーの慣例

| レイヤー | 役割 | 参照例 |
|----------|------|--------|
| migration | スキーマ変更。部分ユニーク等は WHERE 句 | `0004_sender_entries_active_label_unique.sql` |
| domain | 不変条件・`create_new` / `from_persisted` | `domain/sender/sender_entry.rs` |
| repository trait | 非同期 trait + `RepositoryError` | `domain/sender/sender_entry_repository.rs` |
| sqlx impl | トランザクション・エラーマッピング | `infrastructure/sender/sqlx_sender_entry_repository.rs` |
| lib.rs | `*_impl` + Validation / Repository エラー分離 | `create_sender_entry_impl` |
| frontend | Page + hook + types + invoke | `features/sender/SenderEntryEditPage.tsx` |

## 設計書の読み方

1. Issue 本文に設計書パスがあれば最優先で読む
2. `docs/design/<feature>-design.md` を探す
3. ドメイン仕様: `docs/domain/`
4. UI 仕様: `docs/mock-up/`

設計書の「実装タスク」セクションがあればチェックリストとして使う。

## 検証コマンド

| 目的 | コマンド |
|------|----------|
| ESLint | `npm run lint` または `task lint` |
| 型 + フロント build | `npm run build` |
| Rust テスト | `cd src-tauri && cargo test` |
| 型 + Rust check 一式 | `task check` |
| Rust テスト（Taskfile） | `task test` |

特定テストのみ: `cargo test <module_name>`

## ユニットテスト方針

### テスト配置

| 種別 | ファイル | 実行方法 |
|------|----------|----------|
| ドメインロジック | `src-tauri/src/domain/**/*.rs` 内 `#[cfg(test)]` | `cargo test domain::` |
| リポジトリ | `src-tauri/src/infrastructure/**/*_tests.rs` | `cargo test infrastructure::` |
| Tauri コマンド | `src-tauri/src/command_tests.rs` | `cargo test command_tests` |

フロントエンド（Vitest 等）は未導入のため、現状は Rust 側テストを中心とする。

### テスト実装の雛形（command_tests）

```rust
async fn setup_pool() -> SqlitePool {
  let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
  let migrations_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
  sqlx::migrate::Migrator::new(migrations_path).await.unwrap().run(&pool).await.unwrap();
  pool
}

#[tokio::test]
async fn create_xxx_validation_error_on_duplicate() {
  let pool = setup_pool().await;
  // arrange → act → assert
  let err = create_xxx_impl(&pool, dto).await.expect_err("...");
  assert!(err.contains("期待する文言"));
}
```

### カバレッジの目安

| 優先度 | 対象 |
|--------|------|
| 必須 | Issue 受け入れ条件に直結する正常系 |
| 必須 | Validation エラー（ユーザー向け文言の確認） |
| 推奨 | DB 制約・競合・archived 境界 |
| 任意 | 自明な getter / 単純マッピング |

### 失敗時の切り分け

1. **新規テストのみ失敗** → 本番コード or テスト期待値を修正
2. **既存テストも失敗** → regression。本番コードの副作用を疑う
3. **コンパイルエラー** → 型・import・async trait を確認
4. **マイグレーションエラー** → `setup_pool` が最新 migration を適用しているか確認

## 自己レビュー観点の詳細

| 観点 | 確認内容 |
|------|----------|
| 受け入れ条件 | Issue の Done 条件が動作として満たされるか。手動確認手順を頭の中で trace できるか |
| 設計書一致 | API 名・DB カラム・画面構成が設計と食い違わないか |
| 既存パターン | 命名、エラー型、DTO↔domain 変換、invoke 名が近い機能と揃っているか |
| エッジケース | 空入力、not found、archived、競合、ページング境界 |
| テストカバレッジ | 受け入れ条件・Validation・DB 制約がテストで担保されているか |
| 検証 | lint / build / test が通る |
| スコープ | リファクタ・フォーマット・無関係ファイル変更が混ざっていないか |

## よくある実装パターン

### Tauri コマンド

- 公開: `#[tauri::command]` → 内部: `*_impl(pool, ...)`
- Validation エラーはユーザー向け文言、Repository は固定コード + ログ
- テスト可能に `*_impl` を `command_tests` から呼ぶ

### DB

- active 判定: `archived_at IS NULL`（`archived = 0` ではない）
- マイグレーションは新規ファイル追加（既存 migration の編集は避ける）
- 一意制約は部分ユニークインデックス + アプリ側 Validation マッピング

### フロント

- `invoke('snake_case_command', { camelCaseArg: value })`
- ページング: `PaginationControls` + `limit + 1` で hasNext 判定
- ダイアログ: `isOpen` 時のみ内部コンポーネントをマウント（state リセット）
- `useEffect` 内の同期的 `setState` は ESLint エラーになるため避ける

## Linear MCP 操作

| 操作 | ツール |
|------|--------|
| Issue 取得 | `get_issue` |
| ステータス一覧 | `list_issue_statuses` |
| ステータス更新 | `save_issue`（`state`） |

実装完了後のステータス: チームの **Review**（コードレビュー待ち）。Done への変更はユーザー判断。

## 設計スキルとの関係

| スキル | 用途 |
|--------|------|
| `linear-issue-design` | docs/ に設計書を作成 |
| `linear-issue-implement` | 設計に基づきコードを実装 |

設計書が無い Issue は、実装前に設計スキルの実行を提案する。
