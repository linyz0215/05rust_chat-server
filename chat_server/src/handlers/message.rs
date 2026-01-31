use axum::extract::Path;

use axum_extra::headers::Header;
use hyper::HeaderMap;
use tokio::fs::{self, File};

use axum::{
    Extension, Json,
    extract::{Multipart, State},
    response::IntoResponse,
};
use tokio_util::io::ReaderStream;
use tracing::warn;

use crate::{AppError, AppState, ChatFile, User};

pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "send message"
}

pub(crate) async fn list_messages_handler() -> impl IntoResponse {
    "list messages"
}

pub(crate) async fn file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::NotFound("File doesn't exits".to_string()));
    }
    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exits".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.to_string().parse().unwrap());

    Ok((headers, body))
}

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id as u64;
    let base_dir = &state.config.server.base_dir.join(ws_id.to_string());
    let mut files = vec![];
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart field ",);
            continue;
        };
        let file = ChatFile::new(&filename, &data);
        let path = file.path(&base_dir);
        if path.exists() {
            warn!("File {} already exists: {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path should exists")).await?;
            fs::write(path, data).await?;
        }
        files.push(file.url(ws_id));
    }
    Ok(Json(files))
}
