use sqlx::PgPool;

use merc_storage::Storage;

#[derive(Clone)]
pub struct Context {
    pool: PgPool,
}

impl Context {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn storage(&self) -> Storage<'_> {
        Storage::new(&self.pool)
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
