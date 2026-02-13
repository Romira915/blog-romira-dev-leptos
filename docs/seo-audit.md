# SEO / AI検索エンジン対策 監査レポート

**調査日**: 2026-02-13
**対象**: blog.romira.dev（Leptos SSR + Hydration ブログ）

---

## 1. 現状の実装状況

### 1.1 共通レイアウト (`app/src/front/app/shell.rs`)

```html
<html lang="ja">
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="color-scheme" content="dark light" />
  <!-- Google Analytics 4 -->
  <script async src="https://www.googletagmanager.com/gtag/js?id=G-4P5K3SBG1K" />
  <!-- highlight.js -->
  <!-- Leptos CSS / HydrationScripts -->
  <MetaTags />  <!-- ページ別メタタグ挿入ポイント -->
```

### 1.2 App コンポーネント (`app/src/front/app/app_component.rs`)

```html
<Link rel="icon" href="https://blog-romira.imgix.net/.../icon.JPG?w=32&h=32&..." />
```

### 1.3 トップページ (`app/src/front/pages/top_page/top_page_meta.rs`)

| タグ | 値 | 状態 |
|------|----|------|
| `<Title>` | `"Romira's develop blog"` | ✅ |
| `meta[name=description]` | `"Rustaceanによる開発ブログです．..."` | ✅ |
| `meta[property=og:title]` | `"Romira's develop blog"` | ✅ |
| `meta[property=og:description]` | 同上 | ✅ |
| `meta[property=og:type]` | `"website"` | ✅ |
| `meta[property=og:url]` | `"https://blog.romira.dev"` | ✅ |
| `meta[property=og:site_name]` | `"Romira's develop blog"` | ✅ |
| `meta[property=og:image]` | imgix OGP画像URL | ✅ |
| `meta[name=twitter:creator]` | `"@Romira915"` | ✅ |

### 1.4 記事ページ (`app/src/front/pages/article_page/article_page_meta.rs`)

| タグ | 値 | 状態 |
|------|----|------|
| `<Title>` | 記事タイトル（動的） | ✅ |
| `meta[name=description]` | 記事説明文（動的） | ✅ |
| `meta[name=keywords]` | カテゴリをカンマ区切り | ✅ |
| `meta[name=date]` | published_at | ✅ |
| `meta[name=creation_date]` | first_published_at | ✅ |
| `meta[property=og:sitename]` | `"Romira's develop blog"` | ✅ (※ `og:site_name` が正式) |
| `meta[property=og:title]` | 記事タイトル | ✅ |
| `meta[property=og:description]` | 記事説明文 | ✅ |
| `meta[property=og:image]` | OGP画像URL | ✅ |
| `meta[property=og:type]` | `"article"` | ✅ |
| `meta[property=article:published_time]` | published_at | ✅ |
| `meta[property=og:url]` | `https://blog.romira.dev/articles/{id}` | ✅ |
| `meta[name=twitter:card]` | `"summary_large_image"` | ✅ |
| `meta[name=twitter:title]` | 記事タイトル | ✅ |
| `meta[name=twitter:description]` | 記事説明文 | ✅ |
| `meta[name=twitter:image]` | OGP画像URL | ✅ |
| `meta[name=twitter:creator]` | `"@Romira915"` | ✅ |

### 1.5 定数値 (`app/src/constants.rs`)

```
ORIGIN              = "https://blog.romira.dev"
WEB_APP_TITLE       = "Romira's develop blog"
WEB_APP_DESCRIPTION = "Rustaceanによる開発ブログです．技術共有や個人開発の進捗などを発信します．"
WEB_TOP_PAGE_OG_IMAGE_URL = "https://blog-romira.imgix.net/46cea3d7-.../romira'sdevelopblog.png"
ROMIRA_X_URL        = "https://x.com/Romira915"
ROMIRA_GITHUB_URL   = "https://github.com/Romira915"
```

### 1.6 ArticleMetaDto (`app/src/common/dto.rs`)

```rust
pub struct ArticleMetaDto {
    pub(crate) id: RwSignal<String>,
    pub(crate) title: RwSignal<String>,
    pub(crate) description: RwSignal<String>,
    pub(crate) keywords: Vec<RwSignal<String>>,
    pub(crate) og_image_url: RwSignal<String>,
    pub(crate) published_at: RwSignal<String>,
    pub(crate) first_published_at: RwSignal<String>,
}
```

---

## 2. 不足しているタグ一覧

### 2.1 高優先度 — SEO基本要素

#### A. Canonical リンク（全ページ）

**影響**: 重複コンテンツペナルティ回避。検索エンジンが正規URLを認識できない。

| ページ | 追加すべきタグ |
|--------|---------------|
| トップ | `<link rel="canonical" href="https://blog.romira.dev" />` |
| 記事 | `<link rel="canonical" href="https://blog.romira.dev/articles/{id}" />` |

**対象ファイル**: `top_page_meta.rs`, `article_page_meta.rs`

#### B. robots.txt

**影響**: クローラーに対するクロール指令がない。admin ページがインデックスされるリスク。

```
User-agent: *
Allow: /
Disallow: /admin/
Disallow: /preview/
Disallow: /api/

Sitemap: https://blog.romira.dev/sitemap.xml
```

**対象**: 新規エンドポイント `/robots.txt` をAxumに追加

#### C. sitemap.xml

**影響**: 検索エンジン・AI検索クローラーがページを効率的に発見できない。

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://blog.romira.dev</loc>
    <lastmod>2026-02-13</lastmod>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>https://blog.romira.dev/articles/{slug}</loc>
    <lastmod>{published_at}</lastmod>
    <changefreq>monthly</changefreq>
    <priority>0.8</priority>
  </url>
  <!-- 全公開記事を動的生成 -->
</urlset>
```

**対象**: 新規エンドポイント `/sitemap.xml` をAxumに追加（DB/CMSから記事一覧を取得して動的生成）

#### D. JSON-LD 構造化データ

**影響**: Google リッチスニペット非対応。AI検索エンジン（Perplexity, ChatGPT Search, Gemini, Bing Copilot）が記事の著者・日付・カテゴリを正確に抽出できない。

##### トップページ — `WebSite` スキーマ

```json
{
  "@context": "https://schema.org",
  "@type": "WebSite",
  "name": "Romira's develop blog",
  "url": "https://blog.romira.dev",
  "description": "Rustaceanによる開発ブログです．技術共有や個人開発の進捗などを発信します．",
  "author": {
    "@type": "Person",
    "name": "Romira",
    "url": "https://github.com/Romira915"
  },
  "inLanguage": "ja"
}
```

##### 記事ページ — `BlogPosting` スキーマ

```json
{
  "@context": "https://schema.org",
  "@type": "BlogPosting",
  "headline": "{記事タイトル}",
  "description": "{記事説明文}",
  "image": "{OGP画像URL}",
  "datePublished": "{first_published_at (ISO 8601)}",
  "dateModified": "{published_at (ISO 8601)}",
  "author": {
    "@type": "Person",
    "name": "Romira",
    "url": "https://github.com/Romira915"
  },
  "publisher": {
    "@type": "Person",
    "name": "Romira"
  },
  "mainEntityOfPage": {
    "@type": "WebPage",
    "@id": "https://blog.romira.dev/articles/{id}"
  },
  "keywords": ["{カテゴリ1}", "{カテゴリ2}"],
  "inLanguage": "ja"
}
```

**対象ファイル**: `top_page_meta.rs`, `article_page_meta.rs` に `<script type="application/ld+json">` を追加

---

### 2.2 中優先度 — OGP/Twitter Card 補完

#### E. トップページの twitter:card

トップページに `twitter:card` が存在しない。SNS共有時にカード表示されない。

```rust
// top_page_meta.rs に追加
<Meta name="twitter:card" content="summary_large_image" />
<Meta name="twitter:title" content=WEB_APP_TITLE />
<Meta name="twitter:description" content=WEB_APP_DESCRIPTION />
<Meta name="twitter:image" content=WEB_TOP_PAGE_OG_IMAGE_URL />
```

**対象ファイル**: `top_page_meta.rs`

#### F. article:author / article:tag / article:modified_time

記事のOGPメタデータが不完全。

```rust
// article_page_meta.rs に追加
<Meta property="article:author" content="https://blog.romira.dev" />
<Meta property="article:modified_time" content=meta.published_at.get_untracked() />
// 各カテゴリに対して
<Meta property="article:tag" content="{カテゴリ名}" />
```

**対象ファイル**: `article_page_meta.rs`

#### G. og:locale

全ページで言語ロケール情報が未設定。

```rust
<Meta property="og:locale" content="ja_JP" />
```

**対象ファイル**: `top_page_meta.rs`, `article_page_meta.rs`

#### H. og:site_name のタイポ修正

記事ページで `og:sitename`（ハイフンなし）が使われている。正式には `og:site_name`。

```diff
- <Meta property="og:sitename" content=WEB_APP_TITLE />
+ <Meta property="og:site_name" content=WEB_APP_TITLE />
```

**対象ファイル**: `article_page_meta.rs`

---

### 2.3 低優先度 — 補足的改善

#### I. twitter:site

サイト全体のTwitterアカウントが未設定（`twitter:creator` のみ存在）。

```rust
<Meta name="twitter:site" content="@Romira915" />
```

**対象ファイル**: `top_page_meta.rs`, `article_page_meta.rs`

#### J. og:image:width / og:image:height

OGP画像の寸法情報がないため、SNSプラットフォームが画像をキャッシュする際にレンダリングが遅くなる可能性。

```rust
<Meta property="og:image:width" content="1200" />
<Meta property="og:image:height" content="630" />
```

**対象ファイル**: `top_page_meta.rs`, `article_page_meta.rs`（画像サイズが固定であれば）

#### K. theme-color

ブラウザのUI色を指定。モバイル検索結果からの遷移時にブランド色を表示。

```rust
// shell.rs の <head> に追加
<meta name="theme-color" content="#1a1a2e" />
```

**対象ファイル**: `shell.rs`

#### L. dns-prefetch / preconnect

外部リソースへの接続を事前に行い、パフォーマンスを改善。

```html
<link rel="dns-prefetch" href="https://blog-romira.imgix.net" />
<link rel="preconnect" href="https://blog-romira.imgix.net" crossorigin />
<link rel="dns-prefetch" href="https://cdnjs.cloudflare.com" />
<link rel="preconnect" href="https://cdnjs.cloudflare.com" crossorigin />
```

**対象ファイル**: `shell.rs`

---

## 3. 不足タグ全体サマリー

| # | 項目 | 優先度 | 対象ファイル | AI検索への影響 |
|---|------|--------|-------------|---------------|
| A | Canonical リンク | 🔴 高 | top_page_meta.rs, article_page_meta.rs | 中 |
| B | robots.txt | 🔴 高 | 新規 (Axumルート) | 高 |
| C | sitemap.xml | 🔴 高 | 新規 (Axumルート) | 高 |
| D | JSON-LD 構造化データ | 🔴 高 | top_page_meta.rs, article_page_meta.rs | **最高** |
| E | トップ twitter:card | 🟡 中 | top_page_meta.rs | 低 |
| F | article:author/tag/modified_time | 🟡 中 | article_page_meta.rs | 中 |
| G | og:locale | 🟡 中 | top_page_meta.rs, article_page_meta.rs | 低 |
| H | og:sitename → og:site_name | 🟡 中 | article_page_meta.rs | 低 |
| I | twitter:site | 🟢 低 | 両メタファイル | 低 |
| J | og:image:width/height | 🟢 低 | 両メタファイル | 低 |
| K | theme-color | 🟢 低 | shell.rs | なし |
| L | dns-prefetch / preconnect | 🟢 低 | shell.rs | なし |

---

## 4. 実装計画（推奨順序）

### Phase 1: SEO基盤（高優先度）
1. Canonical リンク追加（top_page_meta.rs, article_page_meta.rs）
2. robots.txt エンドポイント追加
3. sitemap.xml 動的生成エンドポイント追加
4. JSON-LD 構造化データ追加（WebSite + BlogPosting）

### Phase 2: メタタグ補完（中優先度）
5. トップページ twitter:card 追加
6. article:author / article:tag / article:modified_time 追加
7. og:locale 追加
8. og:sitename → og:site_name タイポ修正

### Phase 3: 補足改善（低優先度）
9. twitter:site 追加
10. og:image:width / og:image:height 追加
11. theme-color 追加
12. dns-prefetch / preconnect 追加
