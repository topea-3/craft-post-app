# Craft Post App

はがきの宛名・送受信履歴を管理し、印刷レイアウトを扱うデスクトップアプリ（Tauri + React + TypeScript）のリポジトリです。

## リポジトリ構成

- **モノレポ**: アプリ本体・ドキュメント・スクリプトを 1 リポジトリで管理しています。
- **ブランチ**: main（リリース用） / develop（統合用） / feature（作業用）。詳細は [docs/project-setup/repository-and-branch-strategy.md](docs/project-setup/repository-and-branch-strategy.md) を参照してください。
- **ライセンス**: MIT（[LICENSE](LICENSE)、方針は [docs/project-setup/license-decision.md](docs/project-setup/license-decision.md)）。埋め込みフォント（Noto Serif JP / OFL-1.1）は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) を参照。
- **リリース**: [docs/project-setup/release-workflow.md](docs/project-setup/release-workflow.md)（スキル `create-release-pr` + マージ時 CI）。

## ドキュメント

| ドキュメント | 内容 |
|--------------|------|
| [docs/overview/requirements-and-constraints.md](docs/overview/requirements-and-constraints.md) | 要件・制約 |
| [docs/overview/decisions-summary.md](docs/overview/decisions-summary.md) | 技術決定サマリ |
| [docs/project-setup/repository-and-branch-strategy.md](docs/project-setup/repository-and-branch-strategy.md) | リポジトリ・ブランチ・リリース方針 |
| [docs/project-setup/github-setup.md](docs/project-setup/github-setup.md) | GitHub リポジトリ・ブランチ保護の設定手順 |
| [docs/project-setup/dev-setup.md](docs/project-setup/dev-setup.md) | 開発環境の初期構築・起動方法 |
| [docs/project-setup/release-workflow.md](docs/project-setup/release-workflow.md) | リリース運用（スキル + CI） |

## 開発環境の準備

詳細な手順は **[docs/project-setup/dev-setup.md](docs/project-setup/dev-setup.md)** を参照してください。

1. **リポジトリのクローン**

   ```bash
   git clone <repo-url>
   cd craft-post-app
   ```

2. **前提条件**: Node.js（20.x LTS 以上）、Rust（rustup）、Windows の場合は WebView2。

3. **初回セットアップ**

   ```bash
   npm install
   npm run tauri:dev
   ```

4. **ブランチ**: 通常の開発は `develop` から `feature/TOP-XX-...` を切って作業します。

5. **エディタ**: ルートの [.editorconfig](.editorconfig) に従い、インデント・改行・文字コードを統一してください。

開発起動（`npm run tauri:dev`）は identifier `com.topea.craftpost.dev`、本番ビルド（`npm run tauri:build`）は `com.topea.craftpost` を使います。同じ端末でも AppData が分かれるため、開発用データと本番データは衝突しません。

## データの保存場所とバックアップ（v0.1.0）

住所録・送受信履歴は端末ローカルの SQLite（AppData 配下）のみに保存します。**クラウド同期はなく、手動エクスポート／自動バックアップは v0.1.0 では未提供**です。アンインストールや OS のユーザーデータ削除で消える点に注意してください。バックアップ機能は後続バージョンで検討します。

## API ログのデバッグモード（本番ビルド）

Rust 側の `log` 出力をファイルに残す機能です。**開発ビルド**（`tauri:dev` など）ではコンソールに全レベルが出るため、この設定は基本的に **リリース実行ファイル**向けです。デバッグ状態とログフォルダは **永続化されません**（セッション内のみ）。

ログ出力先は **絶対パス**で、ユーザープロファイル・AppData・一時フォルダのいずれかの配下に限ります。

### 起動時に CLI で有効化する

`--api-debug` と **`--api-debug-log-dir` で出力先フォルダの両方**が必要です。

```bash
# Windows の例（パスにスペースがある場合は引用符で囲む）
"Craft Post.exe" --api-debug --api-debug-log-dir "%LOCALAPPDATA%\com.topea.craftpost\logs"
```

```bash
# 等号形式でも指定可能
"Craft Post.exe" --api-debug --api-debug-log-dir=%TEMP%\craft-post-api-logs
```

### フロントから Tauri コマンドで有効化する

**先にログ出力フォルダを指定**し、その後でデバッグを ON にします。フォルダ未指定のまま `set_api_log_debug_enabled(true)` はエラーになります。

```typescript
import { invoke } from '@tauri-apps/api/core'

// 1. 出力フォルダを指定（ユーザープロファイル / AppData / TEMP 配下の絶対パス）
await invoke('set_api_log_debug_directory', {
  directory: 'C:\\Users\\you\\AppData\\Local\\com.topea.craftpost\\logs',
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
- リリースの流れは [docs/project-setup/repository-and-branch-strategy.md#45-リリースの流れラフ](docs/project-setup/repository-and-branch-strategy.md#45-リリースの流れラフ) を参照してください。

## ライセンス

- 本体: [MIT License](LICENSE)（Copyright 2026 Toshiya Takizawa）
- 埋め込みフォント（Noto Serif JP）: [OFL-1.1](third_party/noto-serif-jp/OFL.txt)（詳細は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)）
