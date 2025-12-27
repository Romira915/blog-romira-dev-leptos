-- 公開済み記事テーブル
-- タイムスタンプはUTCで保存、表示時にタイムゾーン変換
CREATE TABLE published_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    description TEXT,
    cover_image_url VARCHAR(512),
    published_at TIMESTAMP DEFAULT (now() AT TIME ZONE 'UTC') NOT NULL,
    created_at TIMESTAMP DEFAULT (now() AT TIME ZONE 'UTC') NOT NULL,
    updated_at TIMESTAMP DEFAULT (now() AT TIME ZONE 'UTC') NOT NULL
);

-- 下書き記事テーブル（新規記事のみ、公開前の下書き）
CREATE TABLE draft_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    description TEXT,
    cover_image_url VARCHAR(512),
    created_at TIMESTAMP DEFAULT (now() AT TIME ZONE 'UTC') NOT NULL,
    updated_at TIMESTAMP DEFAULT (now() AT TIME ZONE 'UTC') NOT NULL
);

-- カテゴリテーブル
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL
);

-- 公開記事-カテゴリ中間テーブル
CREATE TABLE published_article_categories (
    article_id UUID REFERENCES published_articles(id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (article_id, category_id)
);

-- 下書き記事-カテゴリ中間テーブル
CREATE TABLE draft_article_categories (
    article_id UUID REFERENCES draft_articles(id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (article_id, category_id)
);

-- 著者テーブル
CREATE TABLE authors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(512),
    biography TEXT
);

-- インデックス
CREATE INDEX idx_published_articles_slug ON published_articles(slug);
CREATE INDEX idx_published_articles_published_at ON published_articles(published_at DESC);
CREATE INDEX idx_draft_articles_updated_at ON draft_articles(updated_at DESC);
CREATE INDEX idx_categories_slug ON categories(slug);
