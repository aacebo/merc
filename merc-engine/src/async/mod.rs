mod task;

pub use task::*;

///
/// ## Execute
/// represents an async runtime that
/// can spawn/track/manage tasks
///
pub trait Execute: Send + Sync + 'static {
    fn execute<T, H>(&self, handler: H) -> Task<T>
    where
        T: Send + Future + 'static,
        H: FnOnce() -> T;
}
