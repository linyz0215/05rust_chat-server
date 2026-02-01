mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;
use anyhow::Context;
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use tokio::fs;
pub use config::AppConfig;

use core::fmt;
pub use error::AppError;
pub use error::ErrorOutput;
use handlers::*;
use middlewares::set_layer; // verify_token};
pub use models::*;
use sqlx::PgPool;
use std::{ops::Deref, sync::Arc};

use crate::{
    middlewares::verify_token,
    utils::{DecodingKey, EncodingKey},
};


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

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .route("/chats", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chats/{id}",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chats/{id}/messages", get(list_messages_handler))
        .route("/upload", post(upload_handler))
        .route("/files/{ws_id}/{*path}", get(file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);
    Ok(set_layer(app))
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
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("create base_dir failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("Failed to connect to database")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
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
mod test_util {
    use super::*;
    use sqlx::PgPool;
    use sqlx_db_tester::TestPg;
    use sqlx::Executor;
    impl AppState {
        pub async fn try_new_test() -> Result<(TestPg, Self), AppError> {
            let config = AppConfig::load()?;
            let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
            let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
            let server_url = &config.server.db_url;
            println!("server_url: {}", server_url);
            let (tdb,pool) = get_test_pool(Some(server_url)).await;
            Ok((
                tdb,
                Self {
                    inner: Arc::new(AppStateInner {
                        config,
                        ek,
                        dk,
                        pool,
                    }),
                },
            ))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://linyz@localhost/chat".to_string(),
        };
        let tdb = TestPg::new(
            url,
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");
        (tdb, pool) 
    }
}
