mod receiver;
mod sender;

pub use receiver::*;
pub use sender::*;

use tokio::sync::mpsc;

pub struct TokioChannel<T> {
    sender: TokioSender<T>,
    receiver: TokioReceiver<T>,
}

impl<T> TokioChannel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            sender: sender.into(),
            receiver: receiver.into(),
        }
    }

    pub fn bound(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);

        Self {
            sender: sender.into(),
            receiver: receiver.into(),
        }
    }

    pub fn as_sender(&self) -> &TokioSender<T> {
        &self.sender
    }

    pub fn as_receiver(&self) -> &TokioReceiver<T> {
        &self.receiver
    }
}
