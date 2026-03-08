# Craft Post App — 開発環境の初期構築

開発に必要なランタイム・ツールのインストールと、リポジトリの初回セットアップ手順です。

---

## 1. 前提条件（必要なソフトウェア）

| ツール | 推奨バージョン | 用途 |
|--------|----------------|------|
| **Node.js** | 20.x LTS 以上（20.19+ または 22.12+） | フロントエンド（Vite / React）のビルド・開発サーバー |
| **npm** | 10.x 以上（v11.4.0+ 推奨） | パッケージ管理 |
| **Rust** | 最新の安定版（rustup 推奨） | Tauri（Rust）バックエンドのビルド |
| **Windows** | Windows 10 以降 | WebView2 は通常プリインストール。未導入の場合は [Microsoft Edge WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) を参照 |
| **Windows: MSVC ビルドツール** | Visual Studio 2022 Build Tools または Visual Studio 2022 | Rust が Windows でネイティブビルドする際に `link.exe` と C ランタイム（msvcrt.lib 等）を使用するため必須。通常の PowerShell からビルドする場合は「開発者用シェル」で環境変数（LIB, INCLUDE, PATH）を読み込む必要あり（後述）。 |

### 1.1 Node.js のインストール

- [nodejs.org](https://nodejs.org/) から LTS をインストールするか、`nvm` / `fnm` でインストール。
- 確認: `node -v` と `npm -v`

### 1.2 Rust のインストール

- [rustup.rs](https://rustup.rs/) に従い、`rustup` で Rust をインストール。
- 確認: `rustc -V` と `cargo -V`

### 1.3 Windows での MSVC（Visual Studio Build Tools）

- Rust は Windows で **MSVC ツールチェーン**（`link.exe` と C ランタイムライブラリ）を使います。
- **Visual Studio 2022** または **Build Tools for Visual Studio 2022** をインストールし、ワークロードで「C++ によるデスクトップ開発」および **Windows 10/11 SDK** にチェックを入れてください。
- `npm run tauri dev` や `cargo build` は、**開発者用のシェル**（「Developer PowerShell for VS 2022」など）から実行するか、後述の **LNK1104 / msvcrt.lib** 対処に従って環境変数を設定してください。

### 1.4 Windows での WebView2

- Tauri は WebView2 を使用します。Windows 10/11 では多くの場合既に利用可能です。
- 問題がある場合は [WebView2 のドキュメント](https://tauri.app/v1/guides/getting-started/prerequisites#windows) を参照してください。

---

## 2. リポジトリのクローンとブランチ

```bash
git clone <リポジトリURL>
cd craft_post_root
```

通常の開発は `develop` ブランチから `feature/TOP-XX-...` を切って作業します。

```bash
git checkout develop
git pull origin develop
```

---

## 3. 初回セットアップ（依存関係のインストール）

リポジトリルートで次を実行します。

```bash
npm install
```

これでフロントエンド（Vite + React + TypeScript）および Tauri 用の依存関係がインストールされます。

---

## 4. 開発サーバーの起動

### Task を使う場合（推奨・Windows）

Windows で通常の PowerShell からビルドする場合、MSVC の環境変数を読み込むため **Task** と **.env** を使うと便利です。

1. `.env.example` を `.env` にコピーする。
2. `.env` の `VCVARS64_BAT` を、自分の環境の **vcvars64.bat** の絶対パスに書き換える。  
   （例: `E:\ProgramFile2\VisualStudioCommunity\VC\Auxiliary\Build\vcvars64.bat`）
3. 開発サーバー起動: `task dev` または `task`

Task が `tauri dev` の前に `vcvars64.bat` を実行するため、開発者用シェルを開かなくてもビルドできます。

### 直接 npm で起動する場合

```bash
npm run tauri dev
```

- **開発者用シェル**（Developer PowerShell for VS 2022 など）から実行するか、上記の Task + .env を利用してください。
- フロントの Vite 開発サーバー（例: http://localhost:5173）が起動し、Tauri のウィンドウが開きます。
- ホットリロードにより、フロントの変更は自動で反映されます。

---

## 5. その他のよく使うコマンド

| コマンド | 説明 |
|----------|------|
| `task` / `task dev` | 開発サーバー起動（Windows では .env の VCVARS64_BAT を読んでから実行）。 |
| `npm run dev` | フロントのみ（Vite）の開発サーバー。ブラウザで確認する場合に使用。 |
| `npm run build` | フロントの本番ビルド（`dist/` に出力）。 |
| `task build:app` / `npm run tauri build` | Tauri アプリの本番ビルド（インストーラ・実行ファイルを生成）。 |
| `task lint` / `npm run lint` | リンターの実行（設定している場合）。 |

---

## 6. エディタ・フォーマット

- ルートの [.editorconfig](.editorconfig) に従い、インデント・改行・文字コードを統一してください。
- ESLint / Prettier を導入している場合は、保存時フォーマットを有効にすると便利です。

---

## 7. トラブルシューティング

### Rust のビルドエラー

- `cargo clean` の後、再度 `npm run tauri dev` を試してください。
- ツールチェーン: `rustup update` で最新に更新してください。

### フロントのポート競合

- 5173 番ポートが使用中の場合は、`vite.config.ts` で `server.port` を変更できます。

### Windows で WebView2 が見つからない

- [WebView2 常時インストール ランタイム](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section) をインストールしてください。

---

## 変更履歴・メモ

| 日付       | 内容 |
|------------|------|
| 2026-03-08 | 初版。開発環境の前提条件・初回セットアップ・起動方法を記載。 |
