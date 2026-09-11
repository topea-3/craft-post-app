# Tauri Updater + GitHub 公開リポジトリの実現可否（TOP-33）

結論から述べる。

**実現可能。** 公式の `tauri-plugin-updater` と GitHub Releases 上の静的 JSON（`latest.json`）を組み合わせれば、**公開リポジトリ**でもアプリ内アップデートを構築できる。ただし初版（手動ダウンロード配布）とは別に、署名鍵・ビルド成果物・リポジトリ公開範囲の前提を整える必要がある。

---

## 1. 調査結果サマリ

| 観点 | 可否 | メモ |
|------|------|------|
| Tauri v2 Updater プラグイン | ✅ | `tauri-plugin-updater` + `@tauri-apps/plugin-updater` |
| 更新メタデータの置き場 | ✅ | GitHub Releases の `latest.json`（エンドポイント例あり） |
| 公開リポジトリ | ✅ | アセット URL が認証なしで取れるため、静的 JSON 方式と相性が良い |
| 署名 | ✅（必須） | Updater 署名は自作鍵で**無料**。無効化不可 |
| OS コード署名 | 任意 | SmartScreen 等は別問題（有料になりうる） |
| 本リポジトリの現状 CI | △ | 現状は Windows NSIS `.exe` 配布まで。Updater 用アーティファクト / `latest.json` / 署名鍵 secrets は未導入 |

公式ドキュメント:

- [Updater プラグイン](https://v2.tauri.app/plugin/updater/)
- [GitHub 配布パイプライン](https://v2.tauri.app/distribute/pipelines/github/)

## 2. 公開リポジトリで動く理由

Updater は HTTPS でエンドポイントから JSON を取得し、記載されたパッケージ URL をダウンロードして署名検証する。

公開リポジトリなら例えば:

```text
https://github.com/<owner>/<repo>/releases/latest/download/latest.json
```

へアプリからアクセスできる。`tauri-apps/tauri-action` は Release 作成時に updater 用 JSON のアップロードを支援する（`uploadUpdaterJson`）。

**非公開リポジトリ**だと `latest.json` やアセット取得に認証が必要になり、エンドポイント設計が難しくなる（本調査の「公開リポジトリ」前提とは別途設計が必要）。

## 3. 導入時に必要なもの（未実装・将来作業）

1. **鍵ペア生成**（`tauri signer generate`）  
   - 公開鍵 → `tauri.conf.json` の `plugins.updater.pubkey`  
   - 秘密鍵 → GitHub Actions secrets（例: `TAURI_SIGNING_PRIVATE_KEY`）※リポジトリに含めない
2. **設定**  
   - `bundle.createUpdaterArtifacts: true`  
   - `plugins.updater.endpoints` に GitHub Releases の `latest.json` URL
3. **CI 拡張**  
   - 署名付き updater アーティファクトの生成  
   - `latest.json` の Release 添付  
   - 現行の「NSIS `.exe` のみ」方針との両立（updater 用ファイルは追加アセットになる）
4. **アプリ側**  
   - プラグイン初期化と、更新チェック UI / ダイアログ（要件は「必要最小限」）

## 4. 制約・注意

| 項目 | 内容 |
|------|------|
| 署名鍵の喪失 | 既存インストールへの更新発行が事実上できなくなる。バックアップ必須 |
| OS 署名との混同 | Updater 署名 ≠ Authenticode。警告低減は別費用 |
| 初版方針 | [repository-and-branch-strategy.md](./repository-and-branch-strategy.md) どおり、初版は手動入れ替えで可。Updater は後付け可能 |
| プライベート配布 | リポジトリを private のままにする場合は、公開 URL 方式が使えないため別ホストやトークン付き配信を検討 |

## 5. 推奨ロードマップ

1. **今（TOP-33）**: develop→main リリース CI で Windows `.exe` を GitHub Releases に載せる（手動アップデート）
2. **次**: リポジトリを公開する／または更新 JSON を公開 CDN に置く方針を決める
3. **その後**: Updater プラグイン + 署名 secrets + CI で `latest.json` を出す
4. **任意**: Windows コード署名証明書で SmartScreen 対策

## 変更履歴

| 日付 | 内容 |
|------|------|
| 2026-09-12 | TOP-33。公開 GitHub + Tauri Updater が構築可能である旨を文書化。 |
