# 5. API仕様

## 5.1 共通仕様

### Server Function の通信方式

本システムは Leptos の Server Function を使用しており、エンドポイントは Server Function のRPC形式で提供される。

| 入力コーデック | HTTPメソッド | Content-Type |
|--------------|-------------|-------------|
| `GetUrl` | GET | URLクエリパラメータ |
| `Json` | POST | `application/json` |

### エラーレスポンス共通仕様

#### CmsError → HTTPステータスマッピング

CMS層のエラー（`CmsError`）はHTTPステータスコードに以下のルールでマッピングされる。

| CmsError バリアント | HTTPステータス | 説明 |
|-------------------|-------------|------|
| `ValidationError(message)` | 400 Bad Request | バリデーションエラー（メッセージはエラー内容を含む） |
| `NotFound` | 404 Not Found | リソースが見つからない |
| `DatabaseError(sqlx::Error)` | 500 Internal Server Error | DB操作の内部エラー |

**実装箇所**: `app/src/server/http/response.rs` — `cms_error_to_response()`

#### 公開APIのエラー型

公開API（`get_articles_handler`, `get_author_handler`, `get_article_handler`, `get_preview_article_handler`）は、外部サービスのエラーを専用エラー型で返す。

| エラー型 | バリアント | 説明 |
|---------|----------|------|
| `GetArticlesError` | `NewtArticleServiceGetArticles(String)` | Newt CMS記事取得失敗 |
| | `WordPressArticleService(String)` | WordPress記事取得失敗 |
| | `QiitaArticleService(String)` | Qiita記事取得失敗 |
| `GetAuthorError` | `NewtArticleServiceGetAuthor(String)` | Newt CMS著者取得失敗 |
| `GetArticleError` | `NewtArticleServiceGetArticle(String)` | Newt CMS記事取得失敗 |
| | `DatabaseError(String)` | DB操作失敗 |

これらのエラーが発生した場合、HTTPステータス500が設定される。

---

## 5.2 公開API

### 5.2.1 記事一覧取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `get_articles_handler` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | 不要 |

#### 入力

パラメータなし。

ただし、Cookie `features=local` が設定されている場合、DB記事（ローカル記事）も含めて返す。

#### レスポンス型: `Vec<HomePageArticleDto>`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `title` | `RwSignal<String>` | 記事タイトル |
| `thumbnail_url` | `RwSignal<String>` | サムネイル画像URL |
| `src` | `RwSignal<String>` | 記事詳細ページへのパス |
| `category` | `Vec<RwSignal<String>>` | カテゴリ名一覧 |
| `first_published_at` | `RwSignal<String>` | 初回公開日時（表示用文字列） |
| `article_source` | `ArticleSource` | 記事ソース種別 |

`ArticleSource` enum:
- `Newt` — Newt CMS
- `WordPress` — WordPress (PR Times)
- `Qiita` — Qiita
- `Local` — ローカルDB

#### 正常系フロー

1. `features=local` Cookie の有無を判定
2. キャッシュコントロールを設定（features=local: キャッシュ無効化、通常: トップページキャッシュ）
3. Newt CMS / WordPress / Qiita の各外部サービスから記事を並列に取得
4. `features=local` の場合、DB公開記事も取得して追加（取得失敗時はwarnログのみで続行）
5. 全記事を `first_published_at` の降順でソート
6. 200 OK

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| Newt CMS記事取得失敗 | 500 | `GetArticlesError::NewtArticleServiceGetArticles` |
| WordPress記事取得失敗 | 500 | `GetArticlesError::WordPressArticleService` |
| Qiita記事取得失敗 | 500 | `GetArticlesError::QiitaArticleService` |
| DB記事取得失敗（features=local時） | — | ログのみ、処理継続 |

---

### 5.2.2 著者情報取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `get_author_handler` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | 不要 |

#### 入力

パラメータなし。サーバー設定の `ROMIRA_NEWT_AUTHOR_ID` を使用。

#### レスポンス型: `HomePageAuthorDto`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `name` | `RwSignal<String>` | 著者名 |
| `avatar_url` | `RwSignal<String>` | アバター画像URL |
| `description` | `RwSignal<String>` | 自己紹介文 |

#### 正常系フロー

1. キャッシュコントロールを設定
2. Newt CMSから著者情報を取得
3. 200 OK

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| Newt CMS著者取得失敗 | 500 | `GetAuthorError::NewtArticleServiceGetAuthor` |

---

### 5.2.3 記事詳細取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `get_article_handler` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | 不要 |

#### 入力

| パラメータ | 型 | 必須 | 説明 |
|----------|-----|------|------|
| `id` | `String` | ○ | 記事スラッグまたはNewt記事ID |

#### レスポンス型: `ArticleResponse`

```
enum ArticleResponse {
    Found(ArticlePageDto),    // 記事が見つかった
    Redirect(String),         // 別URLへリダイレクト
    NotFound(()),             // 記事が見つからない
}
```

`ArticlePageDto`:

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `article_detail_dto` | `ArticleDetailDto` | 記事本文データ |
| `article_meta_dto` | `ArticleMetaDto` | メタ情報（SEO用） |

`ArticleDetailDto`:

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `title` | `RwSignal<String>` | 記事タイトル |
| `cover_image_url` | `RwSignal<String>` | カバー画像URL |
| `body` | `RwSignal<String>` | 記事本文（HTML） |
| `category` | `Vec<RwSignal<String>>` | カテゴリ名一覧 |
| `first_published_at` | `RwSignal<String>` | 初回公開日時 |

`ArticleMetaDto`:

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `RwSignal<String>` | 記事ID |
| `title` | `RwSignal<String>` | タイトル |
| `description` | `RwSignal<String>` | メタ説明文 |
| `keywords` | `Vec<RwSignal<String>>` | メタキーワード |
| `og_image_url` | `RwSignal<String>` | OG画像URL |
| `published_at` | `RwSignal<String>` | 公開日時 |
| `first_published_at` | `RwSignal<String>` | 初回公開日時 |

#### 正常系フロー（features=local）

1. キャッシュ無効化設定
2. DBから `slug` で公開記事を検索
3. 見つかった場合: `ArticleResponse::Found` を返す（200 OK）
4. 見つからない場合: Newtリダイレクトマッピングを確認
   - マッピングあり: SSR時は301ステータス + `Location`ヘッダ設定、`ArticleResponse::Redirect` を返す
   - マッピングなし: 404（後述の「見つからない場合」へ）

#### 正常系フロー（通常）

1. 記事ページキャッシュ設定
2. Newt CMSから記事を取得
3. 見つかった場合: `ArticleResponse::Found` を返す（200 OK）
4. 見つからない / Newtエラー: 404（後述の「見つからない場合」へ）

#### 見つからない場合

- SSR時: 404ステータスを設定
- `ArticleResponse::NotFound(())` を返す

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| DB記事取得エラー（features=local時） | 500 | `GetArticleError::DatabaseError` |
| Newt CMS記事取得エラー（通常時） | — | warnログのみ、404として扱う |

---

### 5.2.4 プレビュー記事取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `get_preview_article_handler` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | 不要（URLを知っていればアクセス可能） |

#### 入力

| パラメータ | 型 | 必須 | 説明 |
|----------|-----|------|------|
| `id` | `String` | ○ | Newt CMS記事ID |

#### レスポンス型: `Option<ArticlePageDto>`

#### 正常系フロー

1. プレビュー用キャッシュ設定（`no-cache, no-store, private`）
2. Newt CMSからプレビュー記事を取得
3. 見つかった場合: `Some(ArticlePageDto)` を返す（200 OK）
4. 見つからない場合: 404ステータスを設定、`None` を返す

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| Newt CMS記事取得失敗 | 500 | `GetArticleError::NewtArticleServiceGetArticle` |

---

## 5.3 管理API

### 5.3.1 管理記事一覧取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/get_articles` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | UIレベルガードのみ（サーバーサイド認証チェックなし） |

#### 入力

パラメータなし。

#### レスポンス型: `Vec<AdminArticleListItem>`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `String` | 記事ID（UUID文字列） |
| `title` | `String` | 記事タイトル |
| `is_draft` | `bool` | `true`=下書き、`false`=公開 |
| `published_at` | `Option<String>` | 公開日時（`%Y年%m月%d日` 形式）、下書きの場合は `None` |

#### 正常系フロー

1. `AdminArticleService::fetch_all()` で公開記事と下書きを統合取得
2. 各記事を `AdminArticleListItem` に変換（日時は `%Y年%m月%d日` 形式）
3. 200 OK

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| DB取得失敗 | — | `ServerFnError::new(e.to_string())` |

---

### 5.3.2 記事編集データ取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/get_article` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | UIレベルガードのみ |

#### 入力

| パラメータ | 型 | 必須 | 説明 |
|----------|-----|------|------|
| `id` | `String` | ○ | 記事ID（UUID文字列） |

#### レスポンス型: `Option<ArticleEditData>`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `String` | 記事ID |
| `title` | `String` | タイトル |
| `slug` | `String` | スラッグ |
| `body` | `String` | 本文（Markdown） |
| `description` | `Option<String>` | 説明文 |
| `is_draft` | `bool` | `true`=下書き、`false`=公開 |

#### 正常系フロー

1. UUID文字列をパース
2. まず下書きテーブルから検索（`DraftArticleService::fetch_by_id()`）
3. 見つかった場合: `Some(ArticleEditData { is_draft: true, ... })` を返す
4. 下書きになければ公開テーブルから検索（`PublishedArticleService::fetch_by_id_for_admin()`）
5. 見つかった場合: `Some(ArticleEditData { is_draft: false, ... })` を返す
6. 両方見つからない場合: `None` を返す

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| UUID文字列パース失敗 | — | `ServerFnError::new(e.to_string())` |
| DB取得失敗 | — | `ServerFnError::new(e.to_string())` |

---

### 5.3.3 下書き保存

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/save_draft` |
| **入力コーデック** | `Json` (POST) |
| **認証** | UIレベルガードのみ |

#### 入力型: `SaveDraftInput`

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | `String` | ○ | 記事ID（UUID文字列、クライアントで生成） |
| `title` | `String` | ○ | タイトル（バリデーションなし） |
| `slug` | `String` | ○ | スラッグ（バリデーションなし、空文字許可） |
| `body` | `String` | ○ | 本文 |
| `description` | `Option<String>` | — | 説明文 |

#### レスポンス型: `String`（記事ID）

#### 正常系フロー

1. UUID文字列をパース
2. `DraftArticleService::save()` でUpsert（存在しなければ作成、存在すれば更新）
3. 200 OK、記事ID文字列を返す

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| UUID文字列パース失敗 | — | `ServerFnError::new(e.to_string())` |
| DB保存失敗 | — | `ServerFnError::new(e.to_string())` |

**備考**: 下書き保存時はバリデーションなし（Value Object不使用）。タイトル・スラッグ・本文は任意の値を許可。

---

### 5.3.4 公開記事保存

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/save_published` |
| **入力コーデック** | `Json` (POST) |
| **認証** | UIレベルガードのみ |

#### 入力型: `SavePublishedInput`

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | `String` | ○ | 公開記事ID（UUID文字列） |
| `title` | `String` | ○ | タイトル（Value Objectバリデーション適用） |
| `slug` | `String` | ○ | スラッグ（Value Objectバリデーション適用） |
| `body` | `String` | ○ | 本文 |
| `description` | `Option<String>` | — | 説明文 |

#### レスポンス型: `String`（記事ID）

#### 正常系フロー

1. `PublishedArticleTitle::new(input.title)` でタイトルバリデーション
2. `PublishedArticleSlug::new(input.slug)` でスラッグバリデーション
3. UUID文字列をパース
4. `PublishedArticleService::update()` で更新（スラッグ重複チェック含む）
5. 200 OK、記事ID文字列を返す

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| タイトルバリデーション失敗 | 400 | `cms_error_to_response` 経由 |
| スラッグバリデーション失敗 | 400 | `cms_error_to_response` 経由 |
| UUID文字列パース失敗 | — | `ServerFnError::new(e.to_string())` |
| スラッグ重複 | 400 | `cms_error_to_response` 経由 |
| DB更新失敗 | 500 | `cms_error_to_response` 経由 |

---

### 5.3.5 記事公開（下書き→公開）

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/publish_article` |
| **入力コーデック** | `Json` (POST) |
| **認証** | UIレベルガードのみ |

#### 入力型: `PublishArticleInput`

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | `String` | ○ | 下書き記事ID（UUID文字列） |

#### レスポンス型: `String`（公開記事ID）

#### 正常系フロー

1. UUID文字列をパース
2. `DraftArticleService::publish()` を呼び出し:
   1. 下書きを取得（見つからない場合は NotFound エラー）
   2. `PublishedArticleSlug::new()` でスラッグバリデーション
   3. スラッグ重複チェック（published_articles テーブル）
   4. 公開記事を作成（`published_at` = 現在時刻UTC）
   5. カテゴリを下書きから公開にコピー
   6. 下書きを削除
3. 200 OK、公開記事ID文字列を返す

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| UUID文字列パース失敗 | — | `ServerFnError::new(e.to_string())` |
| 下書きが見つからない | 404 | `cms_error_to_response` 経由 |
| スラッグバリデーション失敗 | 400 | `cms_error_to_response` 経由 |
| スラッグ重複 | 400 | `cms_error_to_response` 経由 |
| DB操作失敗 | 500 | `cms_error_to_response` 経由 |

---

### 5.3.6 画像一覧取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/images` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | UIレベルガードのみ |

#### 入力

パラメータなし。

#### レスポンス型: `Vec<ImageDto>`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `String` | 画像ID（UUID文字列） |
| `filename` | `String` | 元のファイル名 |
| `gcs_path` | `String` | GCSオブジェクトパス |
| `mime_type` | `String` | MIMEタイプ |
| `size_bytes` | `i64` | ファイルサイズ（バイト） |
| `width` | `Option<i32>` | 画像幅（ピクセル） |
| `height` | `Option<i32>` | 画像高さ（ピクセル） |
| `alt_text` | `Option<String>` | 代替テキスト |
| `imgix_url` | `String` | imgix CDN URL |
| `created_at` | `String` | 作成日時 |

#### 正常系フロー

1. `ImageService::fetch_all()` で全画像を取得
2. 各画像に `ImgixService::generate_url()` でimgix URLを付与
3. 200 OK

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| DB取得失敗 | — | `ServerFnError::new(e.to_string())` |

---

### 5.3.7 署名付きアップロードURL生成

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/images/upload-url` |
| **入力コーデック** | `Json` (POST) |
| **認証** | UIレベルガードのみ |

#### 入力型: `GenerateUploadUrlInput`

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `filename` | `String` | ○ | アップロードするファイル名 |
| `content_type` | `String` | ○ | MIMEタイプ |
| `size_bytes` | `i64` | ○ | ファイルサイズ（バイト） |

#### レスポンス型: `GenerateUploadUrlResponse`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `upload_url` | `String` | 署名付きPUT URL（有効期限: 15分） |
| `gcs_path` | `String` | GCSオブジェクトパス（DB登録時に使用） |

#### 正常系フロー

1. `ImageService::validate_mime_type()` でMIMEタイプ検証
2. `ImageService::validate_file_size()` でファイルサイズ検証
3. `ImageService::generate_gcs_path()` でGCSパス生成
4. `SigningService::generate_upload_url()` で署名付きURL生成（有効期限: 15分 = 900秒）
5. 200 OK

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| 許可されていないMIMEタイプ | — | `ServerFnError::new(e.to_string())` |
| ファイルサイズ不正（0以下または10MB超過） | — | `ServerFnError::new(e.to_string())` |
| 署名付きURL生成失敗 | — | `ServerFnError::new(e.to_string())` |

---

### 5.3.8 画像登録

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/images` |
| **入力コーデック** | `Json` (POST) |
| **認証** | UIレベルガードのみ |

#### 入力型: `RegisterImageInput`

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `filename` | `String` | ○ | ファイル名 |
| `gcs_path` | `String` | ○ | GCSオブジェクトパス（アップロードURL生成時に取得） |
| `mime_type` | `String` | ○ | MIMEタイプ |
| `size_bytes` | `i64` | ○ | ファイルサイズ |
| `width` | `Option<i32>` | — | 画像幅 |
| `height` | `Option<i32>` | — | 画像高さ |
| `alt_text` | `Option<String>` | — | 代替テキスト |

#### レスポンス型: `RegisterImageResponse`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `String` | 作成された画像ID |
| `imgix_url` | `String` | imgix CDN URL |

#### 正常系フロー

1. `ImageService::create()` で画像を登録:
   1. MIMEタイプバリデーション
   2. ファイルサイズバリデーション
   3. GCSパス重複チェック
   4. DB INSERT
2. `ImgixService::generate_url()` でimgix URL生成
3. 200 OK

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| 許可されていないMIMEタイプ | — | `ServerFnError::new(e.to_string())` |
| ファイルサイズ不正 | — | `ServerFnError::new(e.to_string())` |
| GCSパス重複（既に登録済み） | — | `ServerFnError::new(e.to_string())` |
| DB挿入失敗 | — | `ServerFnError::new(e.to_string())` |

---

### 5.3.9 画像削除

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `admin/images/delete` |
| **入力コーデック** | `Json` (POST) |
| **認証** | UIレベルガードのみ |

#### 入力型: `DeleteImageInput`

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | `String` | ○ | 画像ID（UUID文字列） |

#### レスポンス型: `()`

#### 正常系フロー

1. UUID文字列をパース
2. `ImageService::delete()` でDB削除
3. 200 OK

#### 異常系

| エラーパターン | HTTPステータス | エラー型 |
|-------------|-------------|---------|
| UUID文字列パース失敗 | — | `ServerFnError::new(e.to_string())` |
| DB削除失敗 | — | `ServerFnError::new(e.to_string())` |

**重要な制約**: DB上のレコードのみ削除する。GCS上の実ファイルは削除されず残存する。

---

## 5.4 認証API

### 5.4.1 OAuth設定有無確認

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `auth/configured` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | 不要 |

#### 入力

パラメータなし。

#### レスポンス型: `bool`

#### 正常系フロー

1. `SERVER_CONFIG` の以下3項目がすべて `Some` かを判定:
   - `google_client_id`
   - `google_client_secret`
   - `app_url`
2. 200 OK、`true` または `false`

---

### 5.4.2 認証ユーザー取得

| 項目 | 値 |
|------|-----|
| **エンドポイント** | `auth/me` |
| **入力コーデック** | `GetUrl` (GET) |
| **認証** | 不要（未認証時は `None` を返す） |

#### 入力

パラメータなし。

#### レスポンス型: `Option<AuthUser>`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `email` | `String` | メールアドレス |
| `name` | `Option<String>` | 表示名 |
| `picture` | `Option<String>` | プロフィール画像URL |

#### 正常系フロー

1. セッションから `user` キーで `AuthUser` を取得
2. 200 OK、認証済みなら `Some(AuthUser)`、未認証なら `None`

---

### 5.4.3 OAuth開始

| 項目 | 値 |
|------|-----|
| **パス** | `/auth/google` |
| **メソッド** | GET |
| **認証** | 不要 |
| **種類** | Axumルート（Server Functionではない） |

#### 正常系フロー

1. OAuth2クライアントを構築
2. CSRFトークンを生成しセッションに保存
3. Google認証URLを構築（スコープ: `email`, `profile`）
4. 302 Redirect → Google認証ページ

#### 異常系

| エラーパターン | リダイレクト先 |
|-------------|-------------|
| OAuth未設定（client_id/secret/app_url不足） | `/admin?error=oauth_not_configured` |
| CSRFトークンのセッション保存失敗 | `/admin?error=session_error` |

---

### 5.4.4 OAuthコールバック

| 項目 | 値 |
|------|-----|
| **パス** | `/auth/callback` |
| **メソッド** | GET |
| **認証** | 不要（OAuth フローの一部） |
| **種類** | Axumルート（Server Functionではない） |

#### 入力（クエリパラメータ）

| パラメータ | 型 | 説明 |
|----------|-----|------|
| `code` | `String` | 認証コード（Googleから発行） |
| `state` | `String` | CSRFトークン |

#### 正常系フロー

1. セッションからCSRFトークンを取得し、`state` パラメータと照合
2. CSRFトークンをセッションから削除
3. 認証コードをアクセストークンに交換
4. アクセストークンで Google UserInfo API にリクエスト
5. ユーザー情報（email, name, picture）をセッションに保存
6. 302 Redirect → `/admin`

#### 異常系

| エラーパターン | リダイレクト先 |
|-------------|-------------|
| CSRFトークン不一致 | `/admin?error=csrf_mismatch` |
| OAuth未設定 | `/admin?error=oauth_not_configured` |
| トークン交換失敗 | `/admin?error=token_exchange_failed` |
| UserInfo取得失敗 | `/admin?error=userinfo_failed` |
| UserInfoパース失敗 | `/admin?error=userinfo_parse_failed` |
| セッション保存失敗 | `/admin?error=session_error` |

---

### 5.4.5 ログアウト

| 項目 | 値 |
|------|-----|
| **パス** | `/auth/logout` |
| **メソッド** | GET |
| **認証** | 不要 |
| **種類** | Axumルート |

#### 正常系フロー

1. セッションから `user` キーを削除
2. 302 Redirect → `/`
