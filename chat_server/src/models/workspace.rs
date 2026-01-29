use sqlx::PgPool;

use crate::AppError;

use super::Workspace;
use super::ChatUser;
use crate::User;
impl Workspace {
    pub async fn create(name: &str, user_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
        INSERT INTO workspaces (name, owner_id)
        VALUES ($1, $2)
        RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        // update owner_id in two cases 1) owner_id = 0 2) owner's ws_id = id
        let ws = sqlx::query_as(
            r#"
        UPDATE workspaces
        SET owner_id = $1
        WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as::<_, ChatUser>(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE ws_id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }


}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{models::CreateUser, test_util::get_test_pool};

    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let server_url = "postgres://linyz@localhost/chat";
        let (_tdb,pool) = get_test_pool(Some(server_url)).await;

        let ws = Workspace::create("test1", 0, &pool).await.unwrap();

        let input = CreateUser::new(&ws.name, "linyz1", "linyz12024@shanghaitech.edu.cn", "123456");
        let user = User::create(&input, &pool).await.unwrap();

        assert_eq!(ws.name, "test1");

        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as _, &pool).await.unwrap();

        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }  

     #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let server_url = "postgres://linyz@localhost/chat";
        let (_tdb,pool) = get_test_pool(Some(server_url)).await; 
        let ws = Workspace::find_by_name("foo", &pool).await?;

        assert_eq!(ws.unwrap().name, "foo");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let server_url = "postgres://linyz@localhost/chat";
        let (_tdb,pool) = get_test_pool(Some(server_url)).await;


        let users = Workspace::fetch_all_chat_users(1, &pool).await?;
        assert_eq!(users.len(), 5);
        Ok(())
    }

}