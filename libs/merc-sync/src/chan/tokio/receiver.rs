use std::{
    sync::Arc,
    task::{Context, Poll},
};

use tokio::sync::mpsc;

use crate::chan::{Channel, Receiver, Status};

#[derive(Debug, Clone)]
pub struct TokioReceiver<T: std::fmt::Debug> {
    receiver: Arc<MpscReceiver<T>>,
}

impl<T: std::fmt::Debug> TokioReceiver<T> {
    pub fn new(receiver: Arc<MpscReceiver<T>>) -> Self {
        Self { receiver }
    }
}

impl<T: std::fmt::Debug> std::ops::Deref for TokioReceiver<T> {
    type Target = MpscReceiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl<T: std::fmt::Debug> Channel for TokioReceiver<T> {
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

impl<T: std::fmt::Debug + Send + 'static> Receiver for TokioReceiver<T> {
    type Item = T;

    fn recv(&self) -> Result<Self::Item, crate::chan::error::RecvError> {
        todo!()
    }
}

#[derive(Debug)]
pub enum MpscReceiver<T: std::fmt::Debug> {
    Bound(mpsc::Receiver<T>),
    UnBound(mpsc::UnboundedReceiver<T>),
}

impl<T: std::fmt::Debug> MpscReceiver<T> {
    pub fn status(&self) -> Status {
        if self.is_closed() {
            Status::Closed
        } else {
            Status::Open
        }
    }

    pub fn is_bound(&self) -> bool {
        match self {
            Self::Bound(_) => true,
            _ => false,
        }
    }

    pub fn is_unbound(&self) -> bool {
        match self {
            Self::UnBound(_) => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Bound(v) => v.is_empty(),
            Self::UnBound(v) => v.is_empty(),
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::Bound(v) => v.is_closed(),
            Self::UnBound(v) => v.is_closed(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Bound(v) => v.len(),
            Self::UnBound(v) => v.len(),
        }
    }

    pub fn capacity(&self) -> usize {
        match self {
            Self::Bound(v) => v.capacity(),
            Self::UnBound(v) => v.len(),
        }
    }

    pub fn max_capacity(&self) -> Option<usize> {
        match self {
            Self::Bound(v) => Some(v.max_capacity()),
            Self::UnBound(_) => None,
        }
    }

    pub fn sender_weak_count(&self) -> usize {
        match self {
            Self::Bound(v) => v.sender_weak_count(),
            Self::UnBound(v) => v.sender_weak_count(),
        }
    }

    pub fn sender_strong_count(&self) -> usize {
        match self {
            Self::Bound(v) => v.sender_strong_count(),
            Self::UnBound(v) => v.sender_strong_count(),
        }
    }

    pub fn close(&mut self) {
        match self {
            Self::Bound(v) => v.close(),
            Self::UnBound(v) => v.close(),
        }
    }

    pub async fn recv(&mut self) -> Option<T> {
        match self {
            Self::Bound(v) => v.recv().await,
            Self::UnBound(v) => v.recv().await,
        }
    }

    pub fn try_recv(&mut self) -> Result<T, mpsc::error::TryRecvError> {
        match self {
            Self::Bound(v) => v.try_recv(),
            Self::UnBound(v) => v.try_recv(),
        }
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        match self {
            Self::Bound(v) => v.poll_recv(cx),
            Self::UnBound(v) => v.poll_recv(cx),
        }
    }

    pub fn block_recv(&mut self) -> Option<T> {
        match self {
            Self::Bound(v) => v.blocking_recv(),
            Self::UnBound(v) => v.blocking_recv(),
        }
    }
}

impl<T: std::fmt::Debug> From<mpsc::Receiver<T>> for MpscReceiver<T> {
    fn from(value: mpsc::Receiver<T>) -> Self {
        Self::Bound(value)
    }
}

impl<T: std::fmt::Debug> From<mpsc::UnboundedReceiver<T>> for MpscReceiver<T> {
    fn from(value: mpsc::UnboundedReceiver<T>) -> Self {
        Self::UnBound(value)
    }
}
