# キャッシュ制御テストケース

## TC-401 トップページ: Cache-Controlヘッダの値が仕様通り

| 項目 | 内容 |
|------|------|
| **機能ID** | F-001 |
| **カテゴリ** | 正常系 |

### 前提条件

- サーバーが起動している
- `features` Cookie が未設定（通常モード）

### 手順

1. ブラウザの開発者ツール > ネットワークタブを開く
2. `http://127.0.0.1:3000/` にアクセスする
3. HTMLドキュメントのレスポンスヘッダを確認する

### 期待結果

| ヘッダ | 期待値 |
|--------|--------|
| `Cache-Control` | `no-cache, must-revalidate, max-age=10, stale-while-revalidate=1296000` |
| `CDN-Cache-Control` | `max-age=1296000, stale-while-revalidate=1296000` |

### 備考

- `max-age=10`: ブラウザキャッシュは10秒で期限切れ
- `stale-while-revalidate=1296000`: 15日間（1296000秒）裏でキャッシュ更新しつつ古いキャッシュを返す
- `CDN-Cache-Control`: CDN向けに15日間キャッシュ

---

## TC-402 記事詳細: Cache-Controlヘッダの値が仕様通り

| 項目 | 内容 |
|------|------|
| **機能ID** | F-002 |
| **カテゴリ** | 正常系 |

### 前提条件

- サーバーが起動している
- `features` Cookie が未設定（通常モード）
- Newt CMSに公開済み記事が存在する

### 手順

1. ブラウザの開発者ツール > ネットワークタブを開く
2. `http://127.0.0.1:3000/articles/{slug}` にアクセスする（SSRリクエスト）
3. HTMLドキュメントのレスポンスヘッダを確認する

### 期待結果

| ヘッダ | 期待値 |
|--------|--------|
| `Cache-Control` | `no-cache, must-revalidate, max-age=10, stale-while-revalidate=1296000` |
| `CDN-Cache-Control` | `max-age=1296000, stale-while-revalidate=1296000` |

### 備考

- トップページと同じキャッシュ設定
- 記事ページキャッシュは1リクエスト内で1回のみ呼ばれるため、重複設定防止メカニズム（`CacheControlSet`）は未適用

---

## TC-403 プレビュー: Cache-Control=no-store,private

| 項目 | 内容 |
|------|------|
| **機能ID** | F-003 |
| **カテゴリ** | 正常系 |

### 前提条件

- サーバーが起動している
- Newt CMSに記事が存在する

### 手順

1. ブラウザの開発者ツール > ネットワークタブを開く
2. `http://127.0.0.1:3000/preview/{id}` にアクセスする
3. HTMLドキュメントのレスポンスヘッダを確認する

### 期待結果

| ヘッダ | 期待値 |
|--------|--------|
| `Cache-Control` | `no-cache, must-revalidate, no-store, max-age=0, stale-while-revalidate=0, private` |
| `CDN-Cache-Control` | `no-cache, must-revalidate, no-store, max-age=0, stale-while-revalidate=0, private` |

### 備考

- `no-store`: キャッシュに保存しない
- `private`: CDNキャッシュも禁止
- `max-age=0, stale-while-revalidate=0`: キャッシュ即時無効化
- プレビューは未公開記事の表示であるため、キャッシュを完全に無効化する

---

## TC-404 features=local: 全ページでキャッシュ無効化

| 項目 | 内容 |
|------|------|
| **機能ID** | — |
| **カテゴリ** | 正常系 |

### 前提条件

- サーバーが起動している
- `features=local` Cookie が設定されている
- DBに公開記事が1件以上存在する

### 手順

1. ブラウザの開発者ツール > ネットワークタブを開く
2. `http://127.0.0.1:3000/` にアクセスし、レスポンスヘッダを確認する
3. `http://127.0.0.1:3000/articles/{slug}` にアクセスし、レスポンスヘッダを確認する（`{slug}` はDB公開記事のスラッグ）

### 期待結果

トップページ・記事詳細ページの両方で:

| ヘッダ | 期待値 |
|--------|--------|
| `Cache-Control` | `no-cache, must-revalidate, no-store, max-age=0, stale-while-revalidate=0, private` |
| `CDN-Cache-Control` | `no-cache, must-revalidate, no-store, max-age=0, stale-while-revalidate=0, private` |

### 備考

- `features=local` 時はページ種別に関わらずキャッシュ無効化が最優先（仕様書 12-caching.md §12.4）
- プレビューページ（`/preview/{id}`）は `features=local` の有無に関係なく常にキャッシュ無効化される

---

## TC-405 管理画面: キャッシュ制御ヘッダ未設定

| 項目 | 内容 |
|------|------|
| **機能ID** | — |
| **カテゴリ** | 正常系 |

### 前提条件

- サーバーが起動している
- 管理画面にアクセス可能である（認証済み）

### 手順

1. ブラウザの開発者ツール > ネットワークタブを開く
2. `http://127.0.0.1:3000/admin` にアクセスする
3. HTMLドキュメントのレスポンスヘッダを確認する
4. `http://127.0.0.1:3000/admin/images` にアクセスし、レスポンスヘッダを確認する

### 期待結果

- `Cache-Control` ヘッダが明示的に設定されていない（デフォルト動作）
- `CDN-Cache-Control` ヘッダが設定されていない
- トップページ（TC-401）や記事詳細（TC-402）のようなカスタムキャッシュ値が含まれない

### 備考

- 仕様書（12-caching.md §12.2）: 「管理API（admin/*）にはキャッシュ制御ヘッダを設定しない。デフォルトの動作に従う」
