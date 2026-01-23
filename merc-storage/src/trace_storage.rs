use sqlx::PgPool;

use crate::entity::{Status, Trace};

pub struct TraceStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> TraceStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Trace>, sqlx::Error> {
        sqlx::query_as::<_, Trace>("SELECT * FROM traces WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_request_id(&self, request_id: &str) -> Result<Vec<Trace>, sqlx::Error> {
        sqlx::query_as::<_, Trace>("SELECT * FROM traces WHERE request_id = $1")
            .bind(request_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn get_children(&self, parent_id: uuid::Uuid) -> Result<Vec<Trace>, sqlx::Error> {
        sqlx::query_as::<_, Trace>("SELECT * FROM traces WHERE parent_id = $1")
            .bind(parent_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn create(
        &self,
        parent_id: Option<uuid::Uuid>,
        request_id: Option<String>,
        status: Status,
        status_message: Option<String>,
    ) -> Result<Trace, sqlx::Error> {
        sqlx::query_as::<_, Trace>(
            r#"
            INSERT INTO traces (id, parent_id, request_id, status, status_message, started_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(parent_id)
        .bind(request_id)
        .bind(status)
        .bind(status_message)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: uuid::Uuid,
        status: Status,
        status_message: Option<String>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<Trace>, sqlx::Error> {
        sqlx::query_as::<_, Trace>(
            r#"
            UPDATE traces
            SET status = $2, status_message = $3, ended_at = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(status_message)
        .bind(ended_at)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM traces WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
