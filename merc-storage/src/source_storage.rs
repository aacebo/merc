use sqlx::PgPool;

use crate::entity::Source;

pub struct SourceStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> SourceStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Source>, sqlx::Error> {
        sqlx::query_as::<_, Source>("SELECT * FROM sources WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_scope(&self, scope_id: uuid::Uuid) -> Result<Vec<Source>, sqlx::Error> {
        sqlx::query_as::<_, Source>("SELECT * FROM sources WHERE scope_id = $1")
            .bind(scope_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn get_by_external_id(
        &self,
        external_id: &str,
    ) -> Result<Option<Source>, sqlx::Error> {
        sqlx::query_as::<_, Source>("SELECT * FROM sources WHERE external_id = $1")
            .bind(external_id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn create(&self, source: &Source) -> Result<Source, sqlx::Error> {
        sqlx::query_as::<_, Source>(
            r#"
            INSERT INTO sources (id, scope_id, external_id, type, uri, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#,
        )
        .bind(source.id)
        .bind(source.scope_id)
        .bind(&source.external_id)
        .bind(&source.ty)
        .bind(&source.uri)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sources WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
