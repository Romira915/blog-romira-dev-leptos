# ブログシステム仕様書

## 概要

個人ブログ「blog.romira.dev」のコンテンツ管理システム仕様書。

## ドキュメント構成

| ファイル | 内容 |
|----------|------|
| [01-overview.md](./01-overview.md) | システム概要・アーキテクチャ・技術スタック |
| [02-features.md](./02-features.md) | 機能一覧・各機能の振る舞い仕様 |
| [03-screens.md](./03-screens.md) | 画面仕様・ルーティング・画面遷移 |
| [04-data.md](./04-data.md) | データ仕様・ER図・DTO・変換規則 |
| [05-api.md](./05-api.md) | API仕様・全エンドポイント詳細定義 |
| [06-validation.md](./06-validation.md) | バリデーション・Value Object仕様・エラーメッセージ一覧 |
| [07-external.md](./07-external.md) | 外部連携・各サービスのリクエスト仕様 |
| [08-constraints.md](./08-constraints.md) | 制約事項・セキュリティ・データ整合性 |
| [09-roadmap.md](./09-roadmap.md) | ロードマップ（未実装・改善予定） |
| [10-auth.md](./10-auth.md) | 認証・認可仕様（OAuth2フロー・セッション管理） |
| [11-state-transitions.md](./11-state-transitions.md) | 状態遷移仕様（記事ライフサイクル・画像アップロードフロー） |
| [12-caching.md](./12-caching.md) | キャッシュ制御仕様（ページ種別ごとのCache-Control値） |

## 改訂履歴

| 日付 | バージョン | 内容 |
|------|-----------|------|
| 2026-02-06 | 1.0.0 | 初版作成 |
| 2026-02-07 | 2.0.0 | 全面改訂: API詳細定義、バリデーションマトリクス、状態遷移仕様、認証仕様、キャッシュ仕様を追加。全既存ファイルを実装コードに基づき充実化 |
