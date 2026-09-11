---
name: create-release-pr
description: >-
  リリース用のバージョン bump PR（chore/release-vX.Y.Z → develop）と、
  develop → main のリリース PR を作成する。MAJOR/MINOR/PATCH の選択、
  タグからの次バージョン導出、差分サマリ作成までを行う。ユーザーがリリース、
  バージョン bump、develop→main のリリース PR、vX.Y.Z 公開準備を依頼したときに使う。
disable-model-invocation: true
---

# リリース PR 作成

バージョン bump は **`chore/release-vX.Y.Z` → `develop` の PR** で行い、公開は **`develop` → `main` のリリース PR** で行う。`develop` 直 push はしない（ブランチ保護と両立）。

マージ後のタグ・ビルド・GitHub Releases は CI（`.github/workflows/release.yml`）が担当する。

## 構成

| ファイル | 責務 |
|----------|------|
| `SKILL.md`（本ファイル） | ワークフロー・不変ルール |
| `scripts/next-version.mjs` | 最新 `v*` タグから次バージョンを導出 / `--set` でファイル更新 |
| `references/pr-body-template.md` | CI がパース可能な **develop→main** PR 本文フォーマット |

方針の詳細: `docs/project-setup/release-workflow.md`

## 引数

| 引数 | 必須 | 説明 |
|------|------|------|
| 変更パターン | Yes（Phase A） | **MAJOR** / **MINOR** / **PATCH**（選択） |

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
- **最初に** `git fetch --tags origin` と `git fetch origin main develop` を実行してから導出する
- バージョン導出は **1 回だけ**。ファイル更新は `next-version.mjs --set <next>`（再導出しない）。再計算して `next` が違ったら中断
- バージョン bump は `chore/release-vX.Y.Z` ブランチ経由の **develop 向け PR** のみ（`develop` / `main` 直 push 禁止）
- 公開用 PR の base は必ず `main`、head は必ず `develop`
- サマリは**コーディング知識がない人向け**（専門用語を避け、ユーザー影響を書く）
- develop→main の PR 本文は [references/pr-body-template.md](references/pr-body-template.md) の見出し・`**Version**` ラベルを崩さない（CI 依存）
- commit / push / `gh pr create` は本スキル実行時は許可とみなす
- 作業ツリーが dirty で checkout できない場合は、stash / 別対応を選択肢付きで確認する

## 自律実行ワークフロー

```
Task Progress:
- [ ] Step 0: fetch（tags / main / develop）
- [ ] Step 1: 変更パターン確認
- [ ] Step 2: 次バージョン導出（Script・1 回）
- [ ] Step 3: main…develop 差分からサマリ作成
- [ ] Step 4: chore/release-vX.Y.Z → develop の bump PR
- [ ] Step 5: develop → main のリリース PR（develop に対象版が入っているときのみ）
- [ ] Step 6: 結果報告
```

### Step 0: fetch

```bash
git fetch --tags origin
git fetch origin main develop
```

### Step 1: 変更パターン確認

MAJOR / MINOR / PATCH を確定する（上記テンプレート）。

### Step 2: 次バージョン導出（Script・1 回）

```bash
node .cursor/skills/create-release-pr/scripts/next-version.mjs <major|minor|patch>
```

出力 JSON の `next` / `tag` / `current` を控える。タグが無い場合のみ `current` は `0.0.0`。git 失敗時はスクリプトが非 0 終了するので中断する。

**この `next` を以降の唯一の正とする。** 再実行して値が変わったら中断してユーザーに確認する。

### Step 3: 差分サマリ作成

1. `git log --oneline origin/main..origin/develop` と必要なら `git diff --stat origin/main...origin/develop` を確認
2. 一般向けサマリを日本語で 3〜8 箇条書き程度にまとめる  
   - 良い例: 「宛名の印刷レイアウトを調整し、用紙からはみ出しにくくした」  
   - 避ける例: 「`AddressLayout` の CSS と Tauri コマンドの戻り値を変更」

### Step 4: バージョン bump PR（chore → develop）

`origin/develop` 上の `package.json` / `tauri.conf.json` / `Cargo.toml` が既に Step 2 の `next` と一致している場合は、この Step をスキップして Step 5 へ。

それ以外:

1. `origin/develop` から `chore/release-vX.Y.Z` を作成（既存なら ff 可否を確認）
2. バージョン書き込み（再導出禁止）:

```bash
node .cursor/skills/create-release-pr/scripts/next-version.mjs --set X.Y.Z
```

更新対象: `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml`

3. commit（例: `chore: bump version to X.Y.Z`）し push
4. `gh pr create --base develop --head chore/release-vX.Y.Z`（タイトル例: `chore: bump version to X.Y.Z`）
5. **ここで develop→main はまだ作らない。** bump PR のマージを促し、マージ済みでなければ Step 6 で停止してよい

### Step 5: リリース PR（develop → main）

前提: `origin/develop` の 3 ファイルの version が Step 2 の `next`（またはユーザー指定の公開版）と一致していること。未一致なら bump PR マージ待ちとして停止する。

既に open な `develop` → `main` の PR がある場合は新規作成せず、本文・タイトルの更新可否をユーザーに確認する。

無い場合:

```bash
gh pr create --base main --head develop --title "Release vX.Y.Z" --body-file <pr-body>
```

本文は [references/pr-body-template.md](references/pr-body-template.md) に従う。PowerShell では `--body-file` を使う。

### Step 6: 結果報告

- 導出した `current` → `next`（tag）
- bump PR URL（作成した場合）
- リリース PR URL（作成した場合）／未作成なら「develop マージ後に再実行」と明示
- リリース PR マージ後に CI がタグ・Windows `.exe`・GitHub Release を作る旨

## 完了後（人間作業）

1. **bump PR** を develop へマージ
2. スキル再実行または手動で **develop → main** リリース PR を作成・マージ
3. GitHub Actions の **Release** ワークフロー成功を確認
4. GitHub Releases に `vX.Y.Z` とインストーラ（`.exe`）があることを確認
5. （推奨）main を develop へ同期し、次リリースの差分サマリが累積しないようにする
