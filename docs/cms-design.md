# 自前CMS設計書

## 概要

Newt CMSのサービス終了に伴い、自前でCMS機能を実装する。
既存のWordPress/Qiita連携は維持し、Newtの記事管理機能のみ置き換える。

## 技術スタック

| 項目 | 選定 | 備考 |
|------|------|------|
| データ保存 | PostgreSQL | |
| ORM | SQLx | compile-time checked |
| Markdownパーサー | comrak | サーバー + WASM両対応、GFM完全互換 |
| 画像保存 | GCS | imgix経由で配信 |
| 画像アップロード | 署名付きURL | クライアント→GCS直接アップロード |
| 認証 | Google OAuth | Workspace内部アプリで組織制限 |
| セッション | Cookie + サーバーセッション | tower-sessions + sqlx-store |
| シンタックスハイライト | highlight.js | 既存維持（クライアント側） |

## データベース設計

```sql
-- 記事
CREATE TABLE articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,                    -- Markdown本文
    description TEXT,
    cover_image_url VARCHAR(512),
    draft BOOLEAN DEFAULT true,
    published_at TIMESTAMPTZ,              -- 予約投稿用
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- カテゴリ
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL
);

-- 記事-カテゴリ中間テーブル
CREATE TABLE article_categories (
    article_id UUID REFERENCES articles(id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (article_id, category_id)
);

-- 著者
CREATE TABLE authors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(512),
    biography TEXT
);
```

## ルーティング

### 公開ページ（既存）

| パス | 用途 | 認証 |
|------|------|------|
| `/` | トップページ | 不要 |
| `/articles/:id` | 記事詳細 | 不要 |
| `/preview/:id` | プレビュー | 不要 |

### 管理画面（新規）

| パス | 用途 | 認証 |
|------|------|------|
| `/admin` | 管理画面トップ | 必要 |
| `/admin/articles` | 記事一覧 | 必要 |
| `/admin/articles/new` | 新規作成 | 必要 |
| `/admin/articles/:id` | 編集 | 必要 |

### 認証

| パス | 用途 |
|------|------|
| `/auth/google` | OAuth開始 |
| `/auth/callback` | OAuthコールバック |

## 認証フロー

```
┌──────────┐   1. /admin にアクセス    ┌──────────┐
│  Client  │ ────────────────────────→ │  Server  │
│          │ ←──────────────────────── │          │
│          │   2. Google OAuth へリダイレクト     │
│          │                           └──────────┘
│          │   3. Google ログイン
│          │ ────────────────────────→ ┌──────────┐
│          │ ←──────────────────────── │  Google  │
│          │   4. 認可コード            └──────────┘
│          │
│          │   5. コールバック /auth/callback?code=xxx
│          │ ────────────────────────→ ┌──────────┐
│          │                           │  Server  │
│          │                           │  - コード→トークン交換
│          │                           │  - セッション発行
│          │ ←──────────────────────── │          │
│          │   6. Set-Cookie: session  └──────────┘
└──────────┘
```

- Google Cloud ConsoleでOAuthクライアントを「内部」（Internal）として設定
- Workspace組織内のユーザーのみログイン可能

## 画像アップロードフロー

```
┌──────────┐    1. POST /api/upload/sign     ┌──────────┐
│  Client  │  ─────────────────────────────→ │  Server  │
│          │  ←───────────────────────────── │          │
│          │    2. 署名付きURL返却            │          │
│          │                                 └──────────┘
│          │    3. PUT (画像データ)
│          │  ─────────────────────────────→ ┌──────────┐
│          │  ←───────────────────────────── │   GCS    │
│          │    4. 200 OK                    └──────────┘
│          │
│          │    5. 画像URL保存（記事に紐付け）
└──────────┘
```

## MarkdownエディタUI

### 表示モード

| モード | 表示 | 用途 |
|--------|------|------|
| Split | 左: エディタ / 右: プレビュー | 通常編集 |
| Editor | エディタのみ（全幅） | 集中執筆 |
| Preview | プレビューのみ（全幅） | 確認用 |

### 実装

- `<textarea>` + Leptosコンポーネント
- プレビューはcomrak-wasmでクライアント側変換
- 3モード切り替えボタン

```rust
#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Split,
    Editor,
    Preview,
}

#[component]
fn MarkdownEditor() -> impl IntoView {
    let (content, set_content) = signal(String::new());
    let (mode, set_mode) = signal(ViewMode::Split);

    view! {
        <div class="toolbar">
            <button on:click=move |_| set_mode(ViewMode::Split)>Split</button>
            <button on:click=move |_| set_mode(ViewMode::Editor)>Editor</button>
            <button on:click=move |_| set_mode(ViewMode::Preview)>Preview</button>
        </div>
        <div class="editor-container">
            <Show when=move || mode() != ViewMode::Preview>
                <textarea ... />
            </Show>
            <Show when=move || mode() != ViewMode::Editor>
                <div class="preview" inner_html=move || markdown_to_html(&content()) />
            </Show>
        </div>
    }
}
```

## アーキテクチャ

```
┌─────────────────────────────────────────────────────┐
│                    Leptos App                       │
├─────────────────────────────────────────────────────┤
│  Front (既存)                                        │
│  ├── TopPage      → HomePageArticleDto              │
│  ├── ArticlePage  → ArticlePageDto                  │
│  └── PreviewPage  → ArticlePageDto                  │
├─────────────────────────────────────────────────────┤
│  Server                                             │
│  ├── LocalArticleService (新規) ← Newt置き換え      │
│  ├── WordPressService    (既存)                     │
│  ├── QiitaService        (既存)                     │
│  └── AdminService        (新規) ← 管理画面用        │
├─────────────────────────────────────────────────────┤
│  Admin UI (新規)                                     │
│  ├── /admin/login                                   │
│  ├── /admin/articles      (一覧)                    │
│  ├── /admin/articles/new  (新規作成)                │
│  ├── /admin/articles/:id  (編集)                    │
│  └── Markdown Editor                                │
└─────────────────────────────────────────────────────┘
```

## マイグレーション

- 既存Newt記事: 4件
- 方法: 手動コピペ（Newtエディタから元Markdownをコピー）
- 画像: 既存imgix URLをそのまま使用（GCSに既存）

## インフラ構成

### PostgreSQL

| 環境 | 構成 |
|------|------|
| ローカル開発 | Docker (docker-compose) |
| 本番 | VM直接インストール |

### 画像配信

| 環境 | 構成 |
|------|------|
| 保存先 | GCS (Google Cloud Storage) |
| 配信 | imgix CDN経由 |

## 今後の実装順序

1. PostgreSQL + SQLx セットアップ
2. LocalArticleService 実装（既存DTOへの変換）
3. Google OAuth 認証
4. 管理画面UI（記事一覧、エディタ）
5. 画像アップロード（GCS署名付きURL）
6. 既存記事の手動移行
