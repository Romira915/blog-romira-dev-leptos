-- 画像ライブラリテーブル
-- タイムスタンプはUTCで保存
CREATE TABLE images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename VARCHAR(255) NOT NULL,
    gcs_path VARCHAR(512) NOT NULL UNIQUE,
    mime_type VARCHAR(100) NOT NULL,
    size_bytes BIGINT NOT NULL,
    width INT,
    height INT,
    alt_text VARCHAR(500),
    created_at TIMESTAMP DEFAULT (now() AT TIME ZONE 'UTC') NOT NULL
);

-- インデックス
CREATE INDEX idx_images_created_at ON images(created_at DESC);
