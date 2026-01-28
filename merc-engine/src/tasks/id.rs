use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

///
/// ## TaskId
/// an auto incrementing atomic
/// task identifier
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> &u64 {
        &self.0
    }

    pub fn to_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}
