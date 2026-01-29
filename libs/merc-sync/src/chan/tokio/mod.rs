mod receiver;
mod sender;

pub use receiver::*;
pub use sender::*;

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::chan::{Channel, Status};

#[track_caller]
pub fn open<T: std::fmt::Debug>() -> TokioChannel<T> {
    TokioChannel::new()
}

#[track_caller]
pub fn alloc<T: std::fmt::Debug>(capacity: usize) -> TokioChannel<T> {
    TokioChannel::bound(capacity)
}

#[derive(Debug)]
pub struct TokioChannel<T: std::fmt::Debug> {
    sender: MpscSender<T>,
    receiver: Arc<MpscReceiver<T>>,
}

impl<T: std::fmt::Debug> TokioChannel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            sender: MpscSender::from(sender),
            receiver: Arc::new(MpscReceiver::from(receiver)),
        }
    }

    pub fn bound(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);

        Self {
            sender: MpscSender::from(sender),
            receiver: Arc::new(MpscReceiver::from(receiver)),
        }
    }

    pub fn sender(&self) -> TokioSender<T> {
        TokioSender::new(self.sender.clone())
    }

    pub fn receiver(&self) -> TokioReceiver<T> {
        TokioReceiver::new(Arc::clone(&self.receiver))
    }
}

impl<T: std::fmt::Debug> Clone for TokioChannel<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: Arc::clone(&self.receiver),
        }
    }
}

impl<T: std::fmt::Debug> Channel for TokioChannel<T> {
    fn status(&self) -> Status {
        self.receiver.status()
    }

    fn len(&self) -> usize {
        self.receiver.len()
    }

    fn capacity(&self) -> Option<usize> {
        self.receiver.max_capacity()
    }
}
