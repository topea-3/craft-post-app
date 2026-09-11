# リリース運用（TOP-33 確定版）

`docs/project-setup/repository-and-branch-strategy.md` と Issue TOP-33 の方針に基づく、現行のリリース手順です。

---

## 1. 確定した流れ

```mermaid
flowchart LR
  D[develop] -->|create-release-pr スキル| PR[PR: develop → main]
  PR -->|マージ| M[main]
  M -->|Release CI| T[tag vX.Y.Z]
  T --> B[Windows NSIS .exe ビルド]
  B --> R[GitHub Releases]
```

1. ユーザーが **MAJOR / MINOR / PATCH** を指定する
2. `create-release-pr` スキルが最新 `v*` タグから次バージョンを導出し、バージョンファイルを更新して `develop` に push
3. 同スキルが `main` ← `develop` の PR を作成（本文に Version と一般向け変更サマリ）
4. PR をマージすると **Release** ワークフローが起動し:
   - PR 本文から Version を読み取り `vX.Y.Z` タグを作成
   - Windows 上で NSIS インストーラ（`.exe`）のみビルド
   - GitHub Releases を作成し、`.exe` とサマリを添付

## 2. バージョニング

| 項目 | 内容 |
|------|------|
| 方式 | SemVer `MAJOR.MINOR.PATCH` |
| タグ | `v` プレフィックス（例: `v0.1.0`） |
| 同期先 | `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml` |
| 導出 | `.cursor/skills/create-release-pr/scripts/next-version.mjs`（タグ無し時は `0.0.0` 起点） |

## 3. 成果物

| 項目 | 内容 |
|------|------|
| OS | Windows のみ（初版） |
| 形式 | Tauri NSIS インストーラ（`.exe`） |
| 配布 | GitHub Releases |

## 4. 関連ファイル

| パス | 役割 |
|------|------|
| `.cursor/skills/create-release-pr/` | リリース PR 作成スキル |
| `.github/workflows/release.yml` | マージ時のタグ・ビルド・Release |
| `.github/workflows/ci.yml` | 通常の lint / test / build |
| `docs/project-setup/tauri-updater-feasibility.md` | 自動アップデート可否の調査 |

## 5. 初回リリース（例: v0.1.0）

タグが無い状態では `0.0.0` が起点。`MINOR` を選ぶと `0.1.0`（tag `v0.1.0`）になる。スキル実行 → PR マージ → Actions 確認、の順。

## 変更履歴

| 日付 | 内容 |
|------|------|
| 2026-09-12 | TOP-33。develop→main PR + マージ時 CI を正式運用として記録。 |
