use sqlx::PgPool;

use crate::entity::{Action, Target, TraceAction};

pub struct TraceActionStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> TraceActionStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_trace(
        &self,
        trace_id: uuid::Uuid,
    ) -> Result<Vec<TraceAction>, sqlx::Error> {
        sqlx::query_as::<_, TraceAction>("SELECT * FROM trace_actions WHERE trace_id = $1")
            .bind(trace_id)
            .fetch_all(self.pool)
            .await
    }

    pub async fn get_by_target(
        &self,
        target_id: uuid::Uuid,
        target: Target,
    ) -> Result<Vec<TraceAction>, sqlx::Error> {
        sqlx::query_as::<_, TraceAction>(
            "SELECT * FROM trace_actions WHERE target_id = $1 AND target = $2",
        )
        .bind(target_id)
        .bind(target)
        .fetch_all(self.pool)
        .await
    }

    pub async fn create(
        &self,
        trace_id: uuid::Uuid,
        target_id: uuid::Uuid,
        target: Target,
        action: Action,
    ) -> Result<TraceAction, sqlx::Error> {
        sqlx::query_as::<_, TraceAction>(
            r#"
            INSERT INTO trace_actions (trace_id, target_id, target, action, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#,
        )
        .bind(trace_id)
        .bind(target_id)
        .bind(target)
        .bind(action)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete_by_trace(&self, trace_id: uuid::Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_actions WHERE trace_id = $1")
            .bind(trace_id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
