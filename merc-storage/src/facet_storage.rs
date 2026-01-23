use sqlx::PgPool;

use crate::entity::Facet;

pub struct FacetStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> FacetStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Facet>, sqlx::Error> {
        sqlx::query_as::<_, Facet>("SELECT * FROM facets WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_memory(&self, memory_id: uuid::Uuid) -> Result<Vec<Facet>, sqlx::Error> {
        sqlx::query_as::<_, Facet>("SELECT * FROM facets WHERE memory_id = $1")
            .bind(memory_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn create(&self, facet: &Facet) -> Result<Facet, sqlx::Error> {
        sqlx::query_as::<_, Facet>(
            r#"
            INSERT INTO facets (id, memory_id, type, confidence, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(facet.id)
        .bind(facet.memory_id)
        .bind(&facet.ty)
        .bind(facet.confidence)
        .bind(&facet.data)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, facet: &Facet) -> Result<Option<Facet>, sqlx::Error> {
        sqlx::query_as::<_, Facet>(
            r#"
            UPDATE facets
            SET type = $2, confidence = $3, data = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(facet.id)
        .bind(&facet.ty)
        .bind(facet.confidence)
        .bind(&facet.data)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM facets WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
