mod receiver;
mod sender;

pub use receiver::*;
pub use sender::*;

use tokio::sync::mpsc;

use crate::chan::Channel;

#[derive(Debug)]
pub struct TokioChannel<T: std::fmt::Debug> {
    sender: TokioSender<T>,
    receiver: TokioReceiver<T>,
}

impl<T: std::fmt::Debug> TokioChannel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            sender: TokioSender::from(MpscSender::from(sender)),
            receiver: TokioReceiver::from(MpscReceiver::from(receiver)),
        }
    }

    pub fn bound(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);

        Self {
            sender: TokioSender::from(MpscSender::from(sender)),
            receiver: TokioReceiver::from(MpscReceiver::from(receiver)),
        }
    }

    pub fn as_sender(&self) -> &TokioSender<T> {
        &self.sender
    }

    pub fn to_sender(self) -> TokioSender<T> {
        self.sender
    }

    pub fn as_receiver(&self) -> &TokioReceiver<T> {
        &self.receiver
    }

    pub fn to_receiver(self) -> TokioReceiver<T> {
        self.receiver
    }
}

impl<T: std::fmt::Debug> Channel for TokioChannel<T> {
    fn status(&self) -> super::Status {
        self.receiver.status()
    }
}
