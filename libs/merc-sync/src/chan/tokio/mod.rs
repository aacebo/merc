mod receiver;
mod sender;

use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicUsize, Ordering},
};

pub use receiver::*;
pub use sender::*;

use tokio::sync::mpsc;

use crate::chan::{Channel, Status};

pub fn open<T: std::fmt::Debug>() -> TokioChannel<T> {
    TokioChannel::new()
}

pub fn alloc<T: std::fmt::Debug>(capacity: usize) -> TokioChannel<T> {
    TokioChannel::bound(capacity)
}

#[derive(Debug)]
pub struct TokioChannel<T: std::fmt::Debug> {
    status: AtomicU8,
    length: AtomicUsize,
    capacity: Option<AtomicUsize>,
    sender: MpscSender<T>,
    receiver: MpscReceiver<T>,
}

impl<T: std::fmt::Debug> TokioChannel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            status: AtomicU8::new(Status::Open as u8),
            length: AtomicUsize::new(0),
            capacity: None,
            sender: MpscSender::from(sender),
            receiver: MpscReceiver::from(receiver),
        }
    }

    pub fn bound(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);

        Self {
            status: AtomicU8::new(Status::Open as u8),
            length: AtomicUsize::new(0),
            capacity: Some(AtomicUsize::new(capacity)),
            sender: MpscSender::from(sender),
            receiver: MpscReceiver::from(receiver),
        }
    }

    pub fn sender(self) -> TokioSender<T> {
        TokioSender::new(Arc::new(self))
    }

    pub fn receiver(self) -> TokioReceiver<T> {
        TokioReceiver::new(Arc::new(self))
    }
}

impl<T: std::fmt::Debug> Channel for TokioChannel<T> {
    fn status(&self) -> Status {
        (&self.status).into()
    }

    fn len(&self) -> usize {
        self.length.load(Ordering::Relaxed)
    }

    fn capacity(&self) -> Option<usize> {
        match &self.capacity {
            None => None,
            Some(v) => Some(v.load(Ordering::Relaxed)),
        }
    }
}
