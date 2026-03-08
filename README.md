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

## リリース

- バージョンは **SemVer**（例: `1.0.0`）。タグは `v1.0.0` 形式で **main** に打ちます。
- 配布は **GitHub Releases** で、タグに紐づけて成果物とリリースノートを添付する想定です。
- リリースの流れは [docs/repository-and-branch-strategy.md#45-リリースの流れ](docs/repository-and-branch-strategy.md#45-リリースの流れラフ) を参照してください。

## ライセンス

（未定）
