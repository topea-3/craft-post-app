# GitHub リポジトリ・ブランチ保護の設定手順

Craft Post App を GitHub にプッシュしたあと、リポジトリ設定とブランチ保護を次の手順で行います。  
方針の詳細は [repository-and-branch-strategy.md](./repository-and-branch-strategy.md) を参照してください。

---

## 1. リポジトリの作成とプッシュ

1. GitHub で **New repository** を作成する（例: `craft-post-app`）。  
   - **Initialize with README** は不要（ローカルに既にコミットがあるため）。
2. ローカルでリモートを追加してプッシュする。

   ```bash
   git remote add origin https://github.com/<your-org>/craft-post-app.git
   git push -u origin main
   git push -u origin develop
   ```

3. GitHub の **Settings → General** で **Default branch** を **main** に設定する（未設定の場合）。

---

## 2. ブランチ保護ルール（推奨）

**main** を「常にリリース可能」に保つため、以下の保護を推奨します。

1. **Settings → Branches → Add branch protection rule**
2. **Branch name pattern**: `main`
3. 設定例:
   - **Require a pull request before merging**: 有効  
     - **Require approvals**: 1（個人開発の場合は 0 でも可）
   - **Require status checks to pass before merging**: 有効（CI を設定したあと、該当ワークフローを選択）
   - **Do not allow bypassing the above settings**: 必要に応じて有効（管理者も PR 必須にする場合）
   - **Allow force pushes**: 無効
   - **Allow deletions**: 無効

4. **Create** で保存する。

**develop** についても、直接 push を禁止して PR のみにしたい場合は、同様に `develop` 用のルールを追加できます。

---

## 3. リリースの流れ（運用）

1. 通常の開発は **develop** から **feature/TOP-XX-...** を切り、作業後に **develop** へ PR でマージする。
2. リリース時:
   - **develop** を **main** へ PR でマージする。
   - **main** にタグを打つ（例: `git tag v1.0.0 && git push origin v1.0.0`）。
   - **GitHub Releases** で該当タグのリリースを作成し、ビルド成果物とリリースノートを添付する。
3. （任意）**develop** を **main** と同期する（`develop` で `git merge main`）。

---

## 4. 変更履歴・メモ

| 日付       | 内容 |
|------------|------|
| 2026-03-08 | 初版。GitHub リポジトリ作成・プッシュ・ブランチ保護の手順を記載。 |

