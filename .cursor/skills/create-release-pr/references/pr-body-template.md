# リリース PR 本文テンプレート

`create-release-pr` スキルが `gh pr create` に渡す本文。CI（`.github/workflows/release.yml`）が **Version** と **変更内容のサマリ** をパースする。見出し名・太字ラベルは変更しない。

```markdown
## リリース情報

- **Version**: {{VERSION}}
- **Tag**: {{TAG}}
- **Bump**: {{BUMP}}
- **Base**: `main` ← `develop`

## 変更内容のサマリ

{{SUMMARY}}

## 関連

- リリース用 PR（develop → main）
- スキル: create-release-pr

## チェックリスト

- [x] develop から main へのマージである
- [x] バージョンを SemVer で更新した（package.json / tauri.conf.json / Cargo.toml）
- [x] 変更内容のサマリを一般向けに記載した
- [ ] 動作確認済み（該当する場合）

## 備考

マージ後、Release ワークフローがタグ作成・Windows ビルド・GitHub Releases 公開を行う。
```

### CI が読む箇所

| 項目 | 抽出方法 |
|------|----------|
| バージョン | `- **Version**: X.Y.Z` |
| リリースノート | `## 変更内容のサマリ` から次の `## ` 直前まで |
