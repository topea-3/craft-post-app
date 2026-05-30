# Craft Post App

はがきの宛名・送受信履歴を管理し、印刷レイアウトを扱うデスクトップアプリ（Tauri + React + TypeScript）のリポジトリです。

## リポジトリ構成

- **モノレポ**: アプリ本体・ドキュメント・スクリプトを 1 リポジトリで管理しています。
- **ブランチ**: main（リリース用） / develop（統合用） / feature（作業用）。詳細は [docs/repository-and-branch-strategy.md](docs/repository-and-branch-strategy.md) を参照してください。

## ドキュメント

| ドキュメント | 内容 |
|--------------|------|
| [docs/requirements-and-constraints.md](docs/requirements-and-constraints.md) | 要件・制約 |
| [docs/decisions-summary.md](docs/decisions-summary.md) | 技術決定サマリ |
| [docs/repository-and-branch-strategy.md](docs/repository-and-branch-strategy.md) | リポジトリ・ブランチ・リリース方針 |
| [docs/github-setup.md](docs/github-setup.md) | GitHub リポジトリ・ブランチ保護の設定手順 |
| [docs/dev-setup.md](docs/dev-setup.md) | 開発環境の初期構築・起動方法 |

## 開発環境の準備

詳細な手順は **[docs/dev-setup.md](docs/dev-setup.md)** を参照してください。

1. **リポジトリのクローン**

   ```bash
   git clone <repo-url>
   cd craft_post_root
   ```

2. **前提条件**: Node.js（20.x LTS 以上）、Rust（rustup）、Windows の場合は WebView2。

3. **初回セットアップ**

   ```bash
   npm install
   npm run tauri dev
   ```

4. **ブランチ**: 通常の開発は `develop` から `feature/TOP-XX-...` を切って作業します。

5. **エディタ**: ルートの [.editorconfig](.editorconfig) に従い、インデント・改行・文字コードを統一してください。

## API ログのデバッグモード（本番ビルド）

Rust 側の `log` 出力をファイルに残す機能です。**開発ビルド**（`tauri dev` など）ではコンソールに全レベルが出るため、この設定は基本的に **リリース実行ファイル**向けです。デバッグ状態とログフォルダは **永続化されません**（セッション内のみ）。

### 起動時に CLI で有効化する

`--api-debug` と **`--api-debug-log-dir` で出力先フォルダの両方**が必要です。

```bash
# Windows の例（パスにスペースがある場合は引用符で囲む）
CraftPost.exe --api-debug --api-debug-log-dir "D:\logs\craft-post"
```

```bash
# 等号形式でも指定可能
CraftPost.exe --api-debug --api-debug-log-dir=C:\temp\api-logs
```

### フロントから Tauri コマンドで有効化する

**先にログ出力フォルダを指定**し、その後でデバッグを ON にします。フォルダ未指定のまま `set_api_log_debug_enabled(true)` はエラーになります。

```typescript
import { invoke } from '@tauri-apps/api/core'

// 1. 出力フォルダを指定
await invoke('set_api_log_debug_directory', {
  directory: 'D:\\logs\\craft-post',
})

// 2. デバッグモード ON（この時点のログレベルは DEBUG）
await invoke('set_api_log_debug_enabled', { enabled: true })

// 状態確認
const settings = await invoke<{ debugEnabled: boolean; logDirectory: string | null }>(
  'get_api_log_debug_settings',
)
```

デバッグを止める場合:

```typescript
await invoke('set_api_log_debug_enabled', { enabled: false })
```

## リリース

- バージョンは **SemVer**（例: `1.0.0`）。タグは `v1.0.0` 形式で **main** に打ちます。
- 配布は **GitHub Releases** で、タグに紐づけて成果物とリリースノートを添付する想定です。
- リリースの流れは [docs/repository-and-branch-strategy.md#45-リリースの流れ](docs/repository-and-branch-strategy.md#45-リリースの流れラフ) を参照してください。

## ライセンス

（未定）
