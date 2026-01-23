use sqlx::PgPool;

pub mod entity;

mod facet_storage;
mod memory_source_storage;
mod memory_storage;
mod source_storage;
mod trace_action_storage;
mod trace_storage;

pub use facet_storage::*;
pub use memory_source_storage::*;
pub use memory_storage::*;
pub use source_storage::*;
pub use trace_action_storage::*;
pub use trace_storage::*;

pub struct Storage<'a> {
    pub memories: MemoryStorage<'a>,
    pub facets: FacetStorage<'a>,
    pub sources: SourceStorage<'a>,
    pub memory_sources: MemorySourceStorage<'a>,
    pub traces: TraceStorage<'a>,
    pub trace_actions: TraceActionStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            memories: MemoryStorage::new(pool),
            facets: FacetStorage::new(pool),
            sources: SourceStorage::new(pool),
            memory_sources: MemorySourceStorage::new(pool),
            traces: TraceStorage::new(pool),
            trace_actions: TraceActionStorage::new(pool),
        }
    }
}
