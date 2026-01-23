use sqlx::PgPool;

use crate::entity::MemorySource;

pub struct MemorySourceStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> MemorySourceStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        memory_id: uuid::Uuid,
        source_id: uuid::Uuid,
    ) -> Result<Option<MemorySource>, sqlx::Error> {
        sqlx::query_as::<_, MemorySource>(
            "SELECT * FROM memory_sources WHERE memory_id = $1 AND source_id = $2",
        )
        .bind(memory_id)
        .bind(source_id)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn get_by_memory(
        &self,
        memory_id: uuid::Uuid,
    ) -> Result<Vec<MemorySource>, sqlx::Error> {
        sqlx::query_as::<_, MemorySource>("SELECT * FROM memory_sources WHERE memory_id = $1")
            .bind(memory_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn get_by_source(
        &self,
        source_id: uuid::Uuid,
    ) -> Result<Vec<MemorySource>, sqlx::Error> {
        sqlx::query_as::<_, MemorySource>("SELECT * FROM memory_sources WHERE source_id = $1")
            .bind(source_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn create(
        &self,
        memory_id: uuid::Uuid,
        source_id: uuid::Uuid,
        confidence: f32,
        text: Option<String>,
        hash: String,
        start_offset: i32,
        end_offset: i32,
    ) -> Result<MemorySource, sqlx::Error> {
        sqlx::query_as::<_, MemorySource>(
            r#"
            INSERT INTO memory_sources (memory_id, source_id, confidence, text, hash, start_offset, end_offset)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(memory_id)
        .bind(source_id)
        .bind(confidence)
        .bind(text)
        .bind(hash)
        .bind(start_offset)
        .bind(end_offset)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(
        &self,
        memory_id: uuid::Uuid,
        source_id: uuid::Uuid,
        confidence: f32,
        text: Option<String>,
        hash: String,
        start_offset: i32,
        end_offset: i32,
    ) -> Result<Option<MemorySource>, sqlx::Error> {
        sqlx::query_as::<_, MemorySource>(
            r#"
            UPDATE memory_sources
            SET confidence = $3, text = $4, hash = $5, start_offset = $6, end_offset = $7
            WHERE memory_id = $1 AND source_id = $2
            RETURNING *
            "#,
        )
        .bind(memory_id)
        .bind(source_id)
        .bind(confidence)
        .bind(text)
        .bind(hash)
        .bind(start_offset)
        .bind(end_offset)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(
        &self,
        memory_id: uuid::Uuid,
        source_id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM memory_sources WHERE memory_id = $1 AND source_id = $2")
                .bind(memory_id)
                .bind(source_id)
                .execute(self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }
}
