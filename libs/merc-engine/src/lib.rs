pub mod bench;
mod context;
mod engine;
mod layer;
mod map;
mod options;
pub mod score;

pub use context::*;
pub use engine::*;
pub use layer::*;
pub use map::*;
pub use options::*;

pub trait Value: std::any::Any + std::fmt::Debug {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: std::any::Any + std::fmt::Debug> Value for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<T: Value> AsRef<T> for dyn Value {
    fn as_ref(&self) -> &T {
        self.as_any().downcast_ref().unwrap()
    }
}

impl<T: Value> AsMut<T> for dyn Value {
    fn as_mut(&mut self) -> &mut T {
        self.as_any_mut().downcast_mut().unwrap()
    }
}
