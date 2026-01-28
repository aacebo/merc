mod task;

pub use task::*;

///
/// ## Runtime
/// represents an async runtime that
/// can spawn/track/manage tasks
///
pub trait Runtime: Send + Sync + 'static {
    fn execute<T, H>(&self, handler: H) -> &dyn Task<T>
    where
        T: Send + 'static,
        H: FnOnce() -> T;
}
