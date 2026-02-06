# 6. バリデーション

## 6.1 バリデーションマトリクス（操作×検証項目）

各操作でどのバリデーションが適用されるかを示す。

| バリデーション項目 | 下書き保存 | 公開記事保存 | 記事公開（下書き→公開） | 画像URL生成 | 画像登録 |
|----------------|:--------:|:----------:|:------------------:|:---------:|:------:|
| タイトル必須・200文字以内 | — | ○ | — | — | — |
| スラッグ必須 | — | ○ | ○ | — | — |
| スラッグ文字種 | — | ○ | ○ | — | — |
| スラッグ重複チェック | — | ○ | ○ | — | — |
| MIMEタイプ検証 | — | — | — | ○ | ○ |
| ファイルサイズ検証 | — | — | — | ○ | ○ |
| GCSパス重複チェック | — | — | — | — | ○ |

**重要**: 下書き保存では一切のバリデーションを行わない（タイトル空、スラッグ空、本文空すべて許可）。

---

## 6.2 Value Object 仕様

### 6.2.1 PublishedArticleTitle（公開記事タイトル）

**定義**: `cms/src/value_objects/published_article_title.rs`

| ルール | 条件 | エラーメッセージ |
|--------|------|----------------|
| 必須 | `value.trim().is_empty()` が `true` | `タイトルは必須です` |
| 最大長 | `value.chars().count() > 200` | `タイトルは200文字以内で入力してください` |

**備考**:
- `trim()` で前後空白を除去して判定するため、空白のみの文字列もエラーになる
- 文字数は `chars().count()` で判定（マルチバイト文字対応）
- 最大200文字（200文字ちょうどは有効）

**適用される操作**: 公開記事保存（`admin/save_published`）

---

### 6.2.2 PublishedArticleSlug（公開記事スラッグ）

**定義**: `cms/src/value_objects/published_article_slug.rs`

| ルール | 条件 | エラーメッセージ |
|--------|------|----------------|
| 必須 | `value.is_empty()` が `true` | `スラッグは必須です` |
| 文字種制限 | 半角英小文字 `[a-z]`、数字 `[0-9]`、ハイフン `-`、アンダースコア `_` のみ | `スラッグは半角英小文字、数字、ハイフン、アンダースコアのみ使用できます` |

**備考**:
- `is_empty()` で判定するため、空白のみの文字列は文字種エラーになる
- 大文字英字はエラー、日本語もエラー
- 正規表現で表すと `^[a-z0-9\-_]+$`

**適用される操作**: 公開記事保存（`admin/save_published`）、記事公開（`admin/publish_article`）

---

### 6.2.3 スラッグ重複チェック

**実装**: `PublishedArticleQuery::exists_by_slug()`

| 操作 | 除外条件 | エラーメッセージ |
|------|---------|----------------|
| 記事公開（新規公開） | なし（全レコードと照合） | `このスラッグは既に使用されています` |
| 公開記事保存（更新） | 自身のIDを除外 | `このスラッグは既に使用されています` |

---

## 6.3 画像バリデーション

**定義**: `cms/src/services/image.rs`

### 6.3.1 MIMEタイプ検証

`ImageService::validate_mime_type()` — static メソッド

| 許可MIMEタイプ |
|--------------|
| `image/jpeg` |
| `image/png` |
| `image/gif` |
| `image/webp` |

| ルール | エラーメッセージ |
|--------|----------------|
| 上記以外のMIMEタイプ | `許可されていないファイル形式です: {mime_type}。許可: ["image/jpeg", "image/png", "image/gif", "image/webp"]` |

**適用される操作**: 署名付きアップロードURL生成（`admin/images/upload-url`）、画像登録（`admin/images` POST）

---

### 6.3.2 ファイルサイズ検証

`ImageService::validate_file_size()` — static メソッド

| ルール | 条件 | エラーメッセージ |
|--------|------|----------------|
| 最大サイズ | `size_bytes > 10,485,760` (10MB) | `ファイルサイズが大きすぎます: {size_mb}MB。最大: 10MB` |
| 最小サイズ | `size_bytes <= 0` | `ファイルサイズが不正です` |

**有効範囲**: 1 ≤ `size_bytes` ≤ 10,485,760

**適用される操作**: 署名付きアップロードURL生成（`admin/images/upload-url`）、画像登録（`admin/images` POST）

---

### 6.3.3 GCSパス重複チェック

`ImageQuery::exists_by_gcs_path()` — DB問い合わせ

| ルール | エラーメッセージ |
|--------|----------------|
| 同一 `gcs_path` のレコードが既に存在する | `この画像は既に登録されています` |

**適用される操作**: 画像登録（`admin/images` POST）のみ

---

## 6.4 全エラーメッセージ一覧

| カテゴリ | エラーメッセージ | CmsError バリアント |
|---------|---------------|------------------|
| タイトル | `タイトルは必須です` | `ValidationError` |
| タイトル | `タイトルは200文字以内で入力してください` | `ValidationError` |
| スラッグ | `スラッグは必須です` | `ValidationError` |
| スラッグ | `スラッグは半角英小文字、数字、ハイフン、アンダースコアのみ使用できます` | `ValidationError` |
| スラッグ | `このスラッグは既に使用されています` | `ValidationError` |
| 画像MIME | `許可されていないファイル形式です: {mime_type}。許可: ["image/jpeg", "image/png", "image/gif", "image/webp"]` | `ValidationError` |
| 画像サイズ | `ファイルサイズが大きすぎます: {size_mb}MB。最大: 10MB` | `ValidationError` |
| 画像サイズ | `ファイルサイズが不正です` | `ValidationError` |
| 画像パス | `この画像は既に登録されています` | `ValidationError` |
| 下書き | `Not found`（下書き公開時に下書きが存在しない） | `NotFound` |

---

## 6.5 テストケース対応表

| バリデーション | テストファイル | テストケース名 |
|-------------|-------------|--------------|
| タイトル有効 | `cms/src/value_objects/published_article_title.rs` | `test_有効なタイトルで作成できること` |
| タイトル空文字 | 同上 | `test_空文字列はエラーになること` |
| タイトル空白のみ | 同上 | `test_空白のみはエラーになること` |
| タイトル200文字 | 同上 | `test_200文字ちょうどは有効` |
| タイトル201文字 | 同上 | `test_201文字はエラーになること` |
| スラッグ有効 | `cms/src/value_objects/published_article_slug.rs` | `test_有効なスラッグで作成できること` |
| スラッグ空文字 | 同上 | `test_空文字列スラッグはエラーになること` |
| スラッグ空白のみ | 同上 | `test_空白のみスラッグはエラーになること` |
| スラッグ大文字 | 同上 | `test_大文字を含むスラッグはエラーになること` |
| スラッグ日本語 | 同上 | `test_日本語を含むスラッグはエラーになること` |
| スラッグ数字ハイフン | 同上 | `test_数字とハイフンのみでも有効` |
| スラッグアンダースコア | 同上 | `test_アンダースコアを含むスラッグは有効` |
| 画像MIME許可 | `cms/src/services/image.rs` | `test_validate_mime_typeで許可されたタイプが通ること` |
| 画像MIME拒否 | 同上 | `test_validate_mime_typeで許可されていないタイプがエラーになること` |
| 画像サイズ有効 | 同上 | `test_validate_file_sizeで適切なサイズが通ること` |
| 画像サイズ超過 | 同上 | `test_validate_file_sizeで大きすぎるファイルがエラーになること` |
| 画像サイズ不正 | 同上 | `test_validate_file_sizeで不正なサイズがエラーになること` |
| GCSパス重複 | 同上 | `test_createで重複gcs_pathがエラーになること` |
| スラッグ重複（公開時） | `cms/tests/services_test.rs` | 統合テスト内で検証 |
| CmsError→400 | `app/src/server/http/response.rs` | `test_status_code_from_cms_error_validation_errorの場合bad_requestを返すこと` |
| CmsError→404 | 同上 | `test_status_code_from_cms_error_not_foundの場合not_foundを返すこと` |
| CmsError→500 | 同上 | `test_status_code_from_cms_error_database_errorの場合internal_server_errorを返すこと` |
