mod config;
mod handlers;
mod models;
mod error;
mod utils;
use core::fmt;
use std::{ops::Deref, sync::Arc};
use anyhow::Context;
use axum::{Router, routing::{get, patch, post}};
use handlers::*;
pub use config::AppConfig;
pub use models::User;
pub use error::AppError;
pub use error::ErrorOutput;
use sqlx::PgPool;
#[cfg(test)]
use sqlx_db_tester::TestPg;

use crate::utils::{DecodingKey, EncodingKey};
#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) ek: EncodingKey,
    pub(crate) dk: DecodingKey,
    pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) ->Result<Router,AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route("/chat/{id}", patch(update_chat_handler).delete(delete_chat_handler).post(send_message_handler))
        .route("/chat/{id}/messages", get(list_messages_handler));
    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);
    Ok(app)
}

//appstate.inner.config => appstate.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let pool = PgPool::connect(&config.server.db_url).await.context("Failed to connect to database")?;
        Ok(Self {
            inner: Arc::new(AppStateInner { config, ek, dk, pool }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .field("pool", &self.pool)
            .finish()
    }
}

#[cfg(test)]
impl AppState {
    pub async fn try_new_test(config: AppConfig) -> Result<(TestPg,Self), AppError> {
        use sqlx_db_tester::TestPg;

        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let server_url = &config.server.db_url;
        println!("server_url: {}", server_url);
        let tdb = TestPg::new(
            server_url.to_string(),
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        Ok((tdb, Self {
            inner: Arc::new(AppStateInner { config, ek, dk, pool }),
        }))
    }
}