use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// 画像DTO
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageDto {
    pub id: String,
    pub filename: String,
    pub gcs_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
    pub imgix_url: String,
    pub created_at: String,
}

/// 画像一覧を取得
#[instrument]
#[server(input = GetUrl, endpoint = "admin/images")]
pub async fn get_images_handler() -> Result<Vec<ImageDto>, ServerFnError> {
    use crate::server::contexts::AppState;

    let state = expect_context::<AppState>();
    let service = state.image_service();

    let images = service
        .fetch_all()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let imgix_service = state.imgix_service();

    let dtos: Vec<ImageDto> = images
        .into_iter()
        .map(|img| {
            let imgix_url = imgix_service.generate_url(&img.gcs_path);
            ImageDto {
                id: img.id.to_string(),
                filename: img.filename,
                gcs_path: img.gcs_path,
                mime_type: img.mime_type,
                size_bytes: img.size_bytes,
                width: img.width,
                height: img.height,
                alt_text: img.alt_text,
                imgix_url,
                created_at: img.created_at.to_string(),
            }
        })
        .collect();

    Ok(dtos)
}
