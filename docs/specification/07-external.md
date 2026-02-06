# 7. 外部連携

## 7.1 Newt CMS

### 概要

| 項目 | 値 |
|------|-----|
| **用途** | メイン記事ソース（移行予定あり） |
| **連携方式** | REST API（CDN + APIの2系統） |
| **認証方式** | Bearer Token |
| **実装** | `app/src/server/services/newt.rs` — `NewtArticleService` |

### エンドポイント

| 用途 | URL パターン | メソッド | 認証トークン |
|------|------------|---------|------------|
| 公開記事一覧 | `{CDN_BASE_URL}/blog/article` | GET | `NEWT_CDN_API_TOKEN` |
| 公開記事詳細 | `{CDN_BASE_URL}/blog/article/{id}` | GET | `NEWT_CDN_API_TOKEN` |
| プレビュー記事一覧 | `{API_BASE_URL}/blog/article` | GET | `NEWT_API_TOKEN` |
| プレビュー記事詳細 | `{API_BASE_URL}/blog/article/{id}` | GET | `NEWT_API_TOKEN` |
| 著者情報 | `{CDN_BASE_URL}/blog/author/{id}` | GET | `NEWT_CDN_API_TOKEN` |

**CDN API** (公開用): キャッシュが効き、公開済みコンテンツのみ返す。
**API** (プレビュー用): キャッシュなし、未公開コンテンツも返す。

### レスポンス形式

- 一覧: `NewtArticleCollection` — `{ items: Vec<NewtArticle>, skip, limit, total }`
- 詳細: `NewtArticle` — 記事全体の JSON
- 著者: `Author` — 著者情報の JSON

### エラーハンドリング

| ステータス | 処理 |
|----------|------|
| 200-299 | 正常処理 |
| 404 | 記事/著者なし → `None` 返却 |
| それ以外 | `UnexpectedStatusCode` エラー |

### 環境変数

| 変数名 | 説明 |
|--------|------|
| `NEWT_CDN_API_TOKEN` | CDN用APIトークン（公開コンテンツ取得用） |
| `NEWT_API_TOKEN` | API用トークン（プレビュー含む全コンテンツ取得用） |

---

## 7.2 WordPress (PR Times)

### 概要

| 項目 | 値 |
|------|-----|
| **用途** | PR Times記事の取得 |
| **連携方式** | WordPress REST API v2 |
| **認証方式** | なし（公開API） |
| **実装** | `app/src/server/services/word_press.rs` — `WordPressArticleService` |

### エンドポイント

| 用途 | URL パターン | メソッド |
|------|------------|---------|
| 記事一覧 | `{BASE_URL}/wp-json/wp/v2/posts?author={AUTHOR_ID}` | GET |
| カテゴリ詳細 | `{BASE_URL}/wp-json/wp/v2/categories/{id}` | GET |

### 処理フロー

1. 記事一覧を取得（著者IDでフィルタ）
2. 各記事のカテゴリIDに対して個別にカテゴリ情報をフェッチ（N+1構造）
3. カテゴリ名を記事に紐付け

---

## 7.3 Qiita

### 概要

| 項目 | 値 |
|------|-----|
| **用途** | 技術記事の取得 |
| **連携方式** | Qiita API v2 |
| **認証方式** | Bearer Token |
| **実装** | `app/src/server/services/qiita.rs` — `QiitaArticleService` |

### エンドポイント

| 用途 | URL パターン | メソッド | 認証 |
|------|------------|---------|------|
| 認証ユーザーの記事一覧 | `{BASE_URL}/api/v2/authenticated_user/items` | GET | `QIITA_API_TOKEN` |

### 処理フロー

1. 認証ユーザーの記事一覧を取得
2. 各記事のURLにアクセスしてOG画像URLをスクレイピング
3. OG画像が取得できない場合、ユーザーのプロフィール画像をフォールバック使用

### 環境変数

| 変数名 | 説明 |
|--------|------|
| `QIITA_API_TOKEN` | Qiita APIトークン |

---

## 7.4 Google Cloud Storage (GCS)

### 概要

| 項目 | 値 |
|------|-----|
| **用途** | 画像ファイルの保存 |
| **連携方式** | 署名付きURL（PUT）によるクライアント直接アップロード |
| **認証方式** | サービスアカウントキーJSON |
| **実装** | `app/src/server/services/signing.rs` — `GcsSigningService` |

### 署名付きURL生成

| 項目 | 値 |
|------|-----|
| **HTTPメソッド** | PUT |
| **有効期限** | 15分（900秒） |
| **ヘッダ** | `Content-Type` を署名に含む |
| **ライブラリ** | `google-cloud-storage` / `google-cloud-auth` |

### GCSパス形式

```
{gcs_path_prefix}/images/{uuid_v7}/{filename}
```

例: `prod/images/01936f8a-1234-7000-8000-abcdef123456/photo.jpg`

| セグメント | 説明 |
|----------|------|
| `gcs_path_prefix` | 環境プレフィックス（`dev` / `prod`） |
| `images` | 固定パス |
| `uuid_v7` | UUID v7（タイムスタンプベース、一意性保証） |
| `filename` | 元のファイル名 |

### CORS設定

GCSバケットのCORS設定は Terraform（`romira-arcadia-ops` リポジトリ）で管理。

- 開発環境: `http://127.0.0.1:3000` が許可オリジン
- 本番環境: `https://blog.romira.dev` が許可オリジン

### 環境変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `GCS_BUCKET` | GCSバケット名 | — |
| `GCS_SERVICE_ACCOUNT_KEY_JSON` | サービスアカウントキーJSON（全体） | `{"type":"service_account",...}` |
| `GCS_PATH_PREFIX` | 環境プレフィックス | `dev` / `prod` |

### 非Send問題の回避

`google-cloud-storage` の `SignedUrlBuilder` は内部で非Send型を使用するため、`tokio::task::spawn_blocking` でラップして Leptos Server Function との互換性を確保している。

---

## 7.5 imgix CDN

### 概要

| 項目 | 値 |
|------|-----|
| **用途** | 画像の最適化配信（フォーマット変換、リサイズ） |
| **連携方式** | URL変換（サーバーサイドでURL生成） |
| **実装** | `app/src/server/services/imgix.rs` — `ImgixService` |

### URL生成ルール

| メソッド | URL形式 | 用途 |
|---------|---------|------|
| `generate_url()` | `https://{domain}/{gcs_path}` | 画像一覧のimgix URL |
| `generate_url_with_width()` | `https://{domain}/{gcs_path}?w={width}&auto=format` | レスポンシブ対応サイズ指定 |

### パラメータ

| パラメータ | 説明 |
|----------|------|
| `w` | 出力幅（ピクセル） |
| `auto=format` | ブラウザに最適なフォーマットで自動変換（WebP等） |

### 環境変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `IMGIX_DOMAIN` | imgixドメイン | `blog-romira.imgix.net` |

---

## 7.6 Google OAuth 2.0

### 概要

| 項目 | 値 |
|------|-----|
| **用途** | 管理者認証 |
| **連携方式** | OAuth 2.0 Authorization Code Flow |
| **ライブラリ** | `oauth2` クレート |
| **実装** | `app/src/server/auth.rs` |

### エンドポイント

| 用途 | URL |
|------|-----|
| 認証ページ | `https://accounts.google.com/o/oauth2/v2/auth` |
| トークン交換 | `https://oauth2.googleapis.com/token` |
| ユーザー情報取得 | `https://www.googleapis.com/oauth2/v2/userinfo` |

### リクエスト仕様

**認証開始**: Google認証URLに以下のパラメータを付与してリダイレクト:
- `client_id` — OAuth クライアントID
- `redirect_uri` — `{APP_URL}/auth/callback`
- `scope` — `email profile`
- `state` — CSRFトークン
- `response_type` — `code`

**トークン交換**: `POST https://oauth2.googleapis.com/token`
- `grant_type=authorization_code`
- `code` — 認証コード
- `client_id`, `client_secret`, `redirect_uri`

**ユーザー情報取得**: `GET https://www.googleapis.com/oauth2/v2/userinfo`
- `Authorization: Bearer {access_token}`
- レスポンス: `{ email, name, picture }`

### 環境変数

| 変数名 | 説明 |
|--------|------|
| `GOOGLE_CLIENT_ID` | OAuth2 クライアントID |
| `GOOGLE_CLIENT_SECRET` | OAuth2 クライアントシークレット |
| `APP_URL` | コールバックURL構築用のベースURL |

詳細は [10-auth.md](./10-auth.md) を参照。
