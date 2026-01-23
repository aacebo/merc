use sqlx::PgPool;

use crate::entity::{Memory, Sensitivity};

pub struct MemoryStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> MemoryStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Memory>, sqlx::Error> {
        sqlx::query_as::<_, Memory>("SELECT * FROM memories WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_scope(&self, scope_id: uuid::Uuid) -> Result<Vec<Memory>, sqlx::Error> {
        sqlx::query_as::<_, Memory>("SELECT * FROM memories WHERE scope_id = $1")
            .bind(scope_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn create(
        &self,
        scope_id: uuid::Uuid,
        score: f32,
        confidence: f32,
        importance: f32,
        sensitivity: Sensitivity,
        tags: Vec<String>,
        embedding: Option<Vec<f32>>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Memory, sqlx::Error> {
        sqlx::query_as::<_, Memory>(
            r#"
            INSERT INTO memories (id, scope_id, score, confidence, importance, sensitivity, tags, embedding, expires_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(scope_id)
        .bind(score)
        .bind(confidence)
        .bind(importance)
        .bind(sensitivity)
        .bind(tags)
        .bind(embedding)
        .bind(expires_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: uuid::Uuid,
        score: f32,
        confidence: f32,
        importance: f32,
        sensitivity: Sensitivity,
        tags: Vec<String>,
        embedding: Option<Vec<f32>>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<Memory>, sqlx::Error> {
        sqlx::query_as::<_, Memory>(
            r#"
            UPDATE memories
            SET score = $2, confidence = $3, importance = $4, sensitivity = $5, tags = $6, embedding = $7, expires_at = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(score)
        .bind(confidence)
        .bind(importance)
        .bind(sensitivity)
        .bind(tags)
        .bind(embedding)
        .bind(expires_at)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM memories WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
