# Craft Post App — 決定事項サマリ（TOP-12）

技術スタック・アーキテクチャ検討で決定した内容を 1 箇所にまとめたドキュメントです。  
詳細設計・実装タスクの洗い出しや、新規メンバーのオンボーディングに利用してください。

---

## 1. 採用したデスクトップアプリ技術

| 項目 | 決定内容 |
|------|----------|
| **採用技術** | **Tauri** |
| **概要** | Rust バックエンド + 各 OS のネイティブ WebView（Windows: WebView2、macOS: WebKit）。フロントは Web 技術。 |
| **主な理由** | バイナリ・メモリ・起動時間が軽量、クロスプラットフォーム（Windows 必須・macOS 要検討）を満たす、フロントは Web 技術で既存スキルを活かせる、オフライン・ローカル主体の要件と整合。 |
| **詳細** | [desktop-tech-comparison-and-decision.md](./desktop-tech-comparison-and-decision.md)（TOP-7） |

---

## 2. 使用言語・フレームワーク

| 項目 | 決定内容 |
|------|----------|
| **フロントエンド** | **TypeScript + React** |
| **バックエンド（Tauri シェル）** | **Rust**（Tauri に含まれる） |
| **主な理由** | 型安全と保守性のため TypeScript、コンポーネント化・エコシステムの豊富さのため React。Tauri 公式テンプレートで React+TS が選べ、初期構築がしやすい。 |
| **詳細** | [language-and-framework-decision.md](./language-and-framework-decision.md)（TOP-8） |

---

## 3. データ永続化方式と主要エンティティ

### 3.1 永続化方式

| 項目 | 決定内容 |
|------|----------|
| **主軸** | **SQLite**（1 ファイル = 1 DB） |
| **実装** | Rust 側で rusqlite または sqlx によりアクセス。Tauri コマンド経由でフロントから参照・更新。 |
| **配置** | Tauri のアプリデータディレクトリ配下に SQLite ファイル（例: `craft_post.db`）を 1 つ配置。 |
| **設定など** | キー単位の小さいデータは **tauri-plugin-store** の併用を検討可能（アーキテクチャ設計時に判断）。 |
| **バックアップ・リストア** | 手動エクスポート（JSON/CSV）、ローカル自動バックアップ（詳細は機能設計時）、リストアはインポートで対応。 |
| **詳細** | [data-persistence-decision.md](./data-persistence-decision.md)（TOP-9） |

### 3.2 主要エンティティ（ラフ）

| エンティティ | 説明 |
|--------------|------|
| **Address（住所録）** | 氏名、住所、連絡先、メモ、グループ（任意）。はがきの送付先・差出人として参照される。 |
| **History（はがき送受信履歴）** | 送付日、種別（送信/受信）、宛先・差出人（住所録への参照）、内容メモなど。 |
| **Group（グループ・任意）** | 住所録を分類するためのグループ。住所録エントリが 1 つのグループに属する関係を将来持たせられる。 |

- **Address** → **History**: 1 対多（1 つの住所録エントリに複数の送受信履歴が紐づく）。
- カラム名・正規化の程度は詳細設計で確定。

---

## 4. 印刷・レイアウト方式

| 項目 | 決定内容 |
|------|----------|
| **レイアウト・PDF 生成** | **@react-pdf/renderer**（フロントエンドで React コンポーネントから PDF を生成） |
| **用紙** | 官製はがき 100mm × 148mm。余白 5mm（印刷可能範囲 94mm × 142mm）。 |
| **テンプレート** | **コード内定義**（React コンポーネント + 余白・フォント等の定数）。複数テンプレートやユーザー編集が必要になった段階で SQLite/ファイルを検討。 |
| **プレビュー** | **画面内プレビュー**を実装。印刷用レイアウトと同一のデータ・仕様で表示し、確認後に PDF 生成または印刷ダイアログを呼ぶ。 |
| **詳細** | [printing-layout-decision.md](./printing-layout-decision.md)（TOP-10） |

---

## 5. 全体アーキテクチャ

以下に全体アーキテクチャ図（ラフ）を示す。レイヤと採用技術の対応は表のとおり。

```mermaid
flowchart TB
  subgraph UI["UI レイヤ（画面構成）"]
    direction TB
    A[住所録画面]
    B[はがき送受信履歴画面]
    C[印刷プレビュー / 印刷実行]
    D[設定・バックアップ／リストア]
    A --- B --- C --- D
  end

  subgraph App["アプリケーションロジック層"]
    direction TB
    E[React 状態・UI ロジック]
    F[Tauri コマンド呼び出し]
    G[印刷レイアウト制御]
    E --- F
    E --- G
  end

  subgraph Data["データ永続化層"]
    direction TB
    H[(SQLite\nローカル DB)]
    I[tauri-plugin-store\n設定等・任意]
    H --- I
  end

  subgraph Print["印刷・レイアウトモジュール"]
    direction TB
    J[@react-pdf/renderer\nPDF 生成]
    K[コード内テンプレート\n宛名・レイアウト]
    J --- K
  end

  subgraph Future["将来拡張（クラウド連携ポイント）"]
    L[同期・バックアップ API\n※今回範囲外]
  end

  UI --> App
  App --> Data
  App --> Print
  Future -.->|将来| Data
  Future -.->|将来| App
```

### レイヤと採用技術の対応

| レイヤ／要素 | 採用技術 |
|--------------|----------|
| デスクトップシェル | Tauri（TOP-7） |
| UI・フロント | TypeScript + React（TOP-8） |
| データ永続化 | SQLite（TOP-9） |
| 印刷・レイアウト | @react-pdf/renderer + コード内テンプレート（TOP-10） |

**詳細**（各レイヤの説明・将来拡張など）: [architecture-overview.md](./architecture-overview.md)（TOP-11）

---

## 6. 関連ドキュメント一覧

| ドキュメント | 内容 |
|--------------|------|
| [requirements-and-constraints.md](./requirements-and-constraints.md) | 要件・制約（OS、オフライン、バックアップ等）と技術スタックへの参照 |
| [architecture-overview.md](./architecture-overview.md) | 全体アーキテクチャ図と各レイヤの説明 |
| [desktop-tech-comparison-and-decision.md](./desktop-tech-comparison-and-decision.md) | デスクトップ技術の候補比較・Tauri 採用理由 |
| [language-and-framework-decision.md](./language-and-framework-decision.md) | 言語・フレームワークの候補比較・TypeScript+React 採用理由 |
| [data-persistence-decision.md](./data-persistence-decision.md) | データ永続化の候補比較・SQLite 採用理由・エンティティラフ |
| [printing-layout-decision.md](./printing-layout-decision.md) | 印刷・レイアウトの候補比較・@react-pdf/renderer 採用理由 |
| [repository-and-branch-strategy.md](./repository-and-branch-strategy.md) | リポジトリ構成（モノレポ）・ディレクトリ構成・ブランチ戦略（TOP-13） |

---

## 変更履歴・メモ

| 日付       | 内容 |
|------------|------|
| 2026-03-08 | 初版。TOP-12「決定事項のドキュメント化」に基づき作成。技術スタック・永続化・印刷・アーキテクチャを 1 箇所にまとめた。 |
| 2026-03-08 | 6. 関連ドキュメント一覧に repository-and-branch-strategy.md（TOP-13）を追加。 |
