pub mod error;
mod status;

#[cfg(feature = "tokio")]
pub mod tokio;

pub use status::*;

pub trait Channel {
    fn status(&self) -> Status;
}

pub trait Sender: Channel + Send + Sync + 'static {
    type Item;

    fn send(&self, item: Self::Item) -> Result<(), error::SendError>;
}

pub trait Receiver: Channel + Send + 'static {
    type Item;

    fn recv(&self) -> Result<Self::Item, error::RecvError>;
}
