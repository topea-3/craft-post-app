---
name: create-release-pr
description: >-
  develop から main へのリリース PR を作成する。MAJOR/MINOR/PATCH の選択、
  タグからの次バージョン導出、差分サマリ作成、バージョンファイル更新、
  PR 作成までを行う。ユーザーがリリース、バージョン bump、develop→main の
  リリース PR、vX.Y.Z 公開準備を依頼したときに使う。
disable-model-invocation: true
---

# リリース PR 作成

`develop` → `main` のリリース用 Pull Request を作成する。マージ後のタグ・ビルド・GitHub Releases は CI（`.github/workflows/release.yml`）が担当する。

## 構成

| ファイル | 責務 |
|----------|------|
| `SKILL.md`（本ファイル） | ワークフロー・不変ルール |
| `scripts/next-version.mjs` | 最新 `v*` タグから次バージョンを導出（任意でファイル更新） |
| `references/pr-body-template.md` | CI がパース可能な PR 本文フォーマット |

方針の詳細: `docs/project-setup/release-workflow.md`

## 引数

| 引数 | 必須 | 説明 |
|------|------|------|
| 変更パターン | Yes | **MAJOR** / **MINOR** / **PATCH**（選択） |

未指定なら次の形式で確認してから進む:

```markdown
## 確認: リリースの変更パターン

SemVer のどれで上げますか？
- A: **MAJOR**（破壊的変更）
- B: **MINOR**（後方互換の機能追加）
- C: **PATCH**（バグ修正など）
```

## 不変ルール

- 変更パターン未確定のままバージョン導出・PR 作成をしない
- PR の base は必ず `main`、head は必ず `develop`（別ブランチへのリリース PR は作らない）
- サマリは**コーディング知識がない人向け**（専門用語を避け、ユーザー影響を書く）
- PR 本文は [references/pr-body-template.md](references/pr-body-template.md) の見出し・`**Version**` ラベルを崩さない（CI 依存）
- commit / push / `gh pr create` は本スキル実行時は許可とみなす（ユーザーがスキルで依頼しているため）
- 作業ツリーが dirty でバージョン更新・checkout できない場合は、stash / 別対応を選択肢付きで確認する

## 自律実行ワークフロー

```
Task Progress:
- [ ] Step 1: 変更パターン確認
- [ ] Step 2: 次バージョン導出（Script）
- [ ] Step 3: main…develop 差分からサマリ作成
- [ ] Step 4: バージョンファイル更新・develop へ commit / push
- [ ] Step 5: リリース PR 作成
- [ ] Step 6: 結果報告
```

### Step 1: 変更パターン確認

MAJOR / MINOR / PATCH を確定する（上記テンプレート）。

### Step 2: 次バージョン導出（Script）

リポジトリルートで:

```bash
node .cursor/skills/create-release-pr/scripts/next-version.mjs <major|minor|patch>
```

出力 JSON の `next` / `tag` / `current` を控える。タグが無い場合は `current` は `0.0.0`。

### Step 3: 差分サマリ作成

1. `git fetch origin`
2. `git log --oneline origin/main..origin/develop` と必要なら `git diff --stat origin/main...origin/develop` を確認
3. 一般向けサマリを日本語で 3〜8 箇条書き程度にまとめる  
   - 良い例: 「宛名の印刷レイアウトを調整し、用紙からはみ出しにくくした」  
   - 避ける例: 「`AddressLayout` の CSS と Tauri コマンドの戻り値を変更」

### Step 4: バージョン更新と develop へ反映

1. 最新 `origin/develop` をチェックアウトし ff-only で更新
2. バージョン書き込み:

```bash
node .cursor/skills/create-release-pr/scripts/next-version.mjs <major|minor|patch> --write
```

更新対象: `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml`

3. 変更を commit（メッセージ例: `chore: bump version to X.Y.Z`）し `origin/develop` へ push

### Step 5: リリース PR 作成

既に open な `develop` → `main` の PR がある場合は新規作成せず、本文・タイトルの更新可否をユーザーに確認する。

無い場合:

```bash
gh pr create --base main --head develop --title "Release vX.Y.Z" --body "$(cat <<'EOF'
…templates/pr-body-template.md に従い Version / Tag / Bump / SUMMARY を埋めた本文…
EOF
)"
```

PowerShell の場合はヒアドキュメント相当で本文ファイルを書いて `--body-file` を使ってよい。

### Step 6: 結果報告

- 導出した `current` → `next`（tag）
- サマリ要約
- PR URL
- マージ後に CI がタグ・Windows `.exe`・GitHub Release を作る旨

## 完了後（人間作業）

1. PR をレビューして **main へマージ**
2. GitHub Actions の **Release** ワークフロー成功を確認
3. GitHub Releases に `vX.Y.Z` とインストーラ（`.exe`）があることを確認
