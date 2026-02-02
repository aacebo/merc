pub mod error;
mod result;
mod status;

#[cfg(feature = "tokio")]
pub mod tokio;

pub use status::*;

use async_trait::async_trait;

pub trait Channel {
    fn status(&self) -> Status;
    fn len(&self) -> usize;
    fn capacity(&self) -> Option<usize>;
}

pub trait Sender: Channel + Send + Sync + 'static {
    type Item: Send;

    fn send(&self, item: Self::Item) -> Result<(), error::SendError>;
}

#[async_trait]
pub trait AsyncSender: Sender {
    async fn send_async(&self, item: Self::Item) -> Result<(), error::SendError> {
        self.send(item)
    }
}

#[async_trait]
impl<T: Sender + ?Sized> AsyncSender for T {}

pub trait Receiver: Channel + Send {
    type Item: Send;

    fn close(&mut self);
    fn recv(&mut self) -> Result<Self::Item, error::RecvError>;
    fn recv_poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<Self::Item, error::RecvError>>;
}

#[async_trait]
pub trait AsyncReceiver: Receiver {
    async fn recv_async(&mut self) -> Result<Self::Item, error::RecvError> {
        self.recv()
    }
}
