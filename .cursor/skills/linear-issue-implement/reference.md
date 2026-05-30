# Linear Issue 実装 — リファレンス

手順・フローは [SKILL.md](SKILL.md)。本ファイルは実装作業の参照資料のみ。

## プロジェクト構成

```
src-tauri/
├── migrations/
└── src/
    ├── domain/
    ├── infrastructure/
    ├── lib.rs
    └── command_tests.rs

src/
├── features/
└── components/
```

## レイヤー慣例

| レイヤー | 役割 | 参照例 |
|----------|------|--------|
| migration | スキーマ変更。部分ユニークは WHERE 句 | `0004_sender_entries_active_label_unique.sql` |
| domain | 不変条件・`create_new` / `from_persisted` | `domain/sender/sender_entry.rs` |
| repository trait | 非同期 trait + `RepositoryError` | `domain/sender/sender_entry_repository.rs` |
| sqlx impl | トランザクション・エラーマッピング | `infrastructure/sender/sqlx_sender_entry_repository.rs` |
| lib.rs | `*_impl` + Validation / Repository エラー分離 | `create_sender_entry_impl` |
| frontend | Page + hook + types + invoke | `features/sender/SenderEntryEditPage.tsx` |

## 設計書の読み方

1. Issue 本文の設計書パス
2. `docs/design/<feature>-design.md`
3. `docs/domain/` / `docs/mock-up/`

「実装タスク」「テスト方針」セクションをチェックリストとして使う。

## 検証コマンド

| 目的 | コマンド |
|------|----------|
| ESLint | `npm run lint` / `task lint` |
| 型 + フロント build | `npm run build` |
| Rust テスト | `cd src-tauri && cargo test` |
| 一式 | `task check` / `task test` |

特定モジュール: `cargo test <module_name>`

## ユニットテスト

### 手順（Step 6）

```
- [ ] 6a. テストケース洗い出し
- [ ] 6b. テスト実装
- [ ] 6c. テスト実行
- [ ] 6d. 結果確認
- [ ] 6e. 失敗時は修正して 6c へ
```

### 6a. 洗い出し（最低限）

- 正常系
- Validation エラー（not found・archived 等）
- DB 制約・競合（UNIQUE、部分インデックス）
- 境界値（空配列、ページング、0 件）

### 6b. 配置

| 対象 | 配置 |
|------|------|
| ドメイン | 同一ファイル `#[cfg(test)] mod tests` |
| リポジトリ | `infrastructure/*/*_tests.rs` |
| Tauri コマンド | `command_tests.rs`（`*_impl` を直接呼ぶ） |

- `setup_pool()` / `sample_*()` を再利用
- 命名: `#[tokio::test] async fn <action>_<expected_outcome>()`
- 自明なテストは書かない

### 6c–6d. 実行・確認

```bash
cd src-tauri && cargo test <module_or_test_name>   # 推奨: 変更範囲に絞る
cd src-tauri && cargo test                         # 全体
```

- 全件 passed、新規テストが filtered out されていないこと

### 6e. 失敗時の切り分け

| 状況 | 対応 |
|------|------|
| 本番コードのバグ | Step 5 に戻る |
| テストの期待値・セットアップ誤り | テスト修正 → 6c |
| 既存テストも失敗（regression） | 本番コードの副作用を修正 |

### 雛形（command_tests）

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
  let err = create_xxx_impl(&pool, dto).await.expect_err("...");
  assert!(err.contains("期待する文言"));
}
```

### カバレッジの目安

| 優先度 | 対象 |
|--------|------|
| 必須 | 受け入れ条件の正常系 |
| 必須 | Validation エラー（文言確認） |
| 推奨 | DB 制約・archived 境界 |
| 任意 | 自明な getter / マッピング |

フロント（Vitest 等）は未導入。現状は Rust テスト中心。

## 自己レビュー

ワークフロー Step 7 の観点に対応するチェックリスト。

```
- [ ] Issue 受け入れ条件を満たす
- [ ] 設計書・docs と一致
- [ ] 既存パターン・レイヤー構成に沿う
- [ ] エッジケース・エラー処理が適切
- [ ] ユニットテストが要件・エッジケースをカバー
- [ ] lint / build / test 成功
- [ ] スコープ外変更・過剰実装なし
```

| 観点 | 確認内容 |
|------|----------|
| 受け入れ条件 | Done 条件を動作として trace できるか |
| 設計書一致 | API・DB・UI が設計と一致するか |
| 既存パターン | 命名、エラー型、invoke 名が揃っているか |
| エッジケース | 空入力、not found、archived、競合、ページング |
| テストカバレッジ | 受け入れ条件・Validation・DB 制約がテストで担保されているか |
| 検証 | lint / build / test 成功 |
| スコープ | 無関係な変更が混ざっていないか |

## 実装パターン（クイックリファレンス）

**Tauri**: `#[tauri::command]` → `*_impl` / Validation は文言、Repository は固定コード + ログ

**DB**: `archived_at IS NULL` / マイグレーションは新規ファイル / 部分ユニーク + Validation マッピング

**フロント**: `invoke('snake_case', { camelCase })` / `PaginationControls` + `limit+1` / ダイアログは `isOpen` 時のみマウント / `useEffect` 内の同期 `setState` 禁止

## 関連スキル

| スキル | 用途 |
|--------|------|
| `linear-issue-design` | docs/ に設計書を作成 |
| `linear-issue-implement` | 本スキル — 設計に基づき実装 |

設計書が無い Issue は、実装前に設計スキルの実行を提案する。
