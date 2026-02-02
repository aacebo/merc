use std::task::{Context, Poll};

use tokio::sync::mpsc;

use crate::chan::{Channel, Receiver, Status, error::RecvError};

pub struct TokioReceiver<T> {
    receiver: MpscReceiver<T>,
}

impl<T> std::fmt::Debug for TokioReceiver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioReceiver")
            .field("receiver", &self.receiver)
            .finish()
    }
}

impl<T> TokioReceiver<T> {
    pub fn new(receiver: MpscReceiver<T>) -> Self {
        Self { receiver }
    }
}

impl<T> Channel for TokioReceiver<T> {
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

impl<T: Send + 'static> Receiver for TokioReceiver<T> {
    type Item = T;

    fn close(&mut self) {
        if self.receiver.is_closed() {
            return;
        }

        while !self.receiver.is_empty() {
            let _ = self.receiver.recv();
        }

        self.receiver.close();
    }

    fn recv(&mut self) -> Result<Self::Item, RecvError> {
        match self.receiver.block_recv() {
            None => {
                if self.status().is_closed() {
                    Err(RecvError::Closed)
                } else {
                    Err(RecvError::Empty)
                }
            }
            Some(v) => Ok(v),
        }
    }

    fn recv_poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<Self::Item, RecvError>> {
        match self.receiver.poll_recv(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(v) => Poll::Ready(match v {
                None => {
                    if self.status().is_closed() {
                        Err(RecvError::Closed)
                    } else {
                        Err(RecvError::Empty)
                    }
                }
                Some(v) => Ok(v),
            }),
        }
    }
}

pub enum MpscReceiver<T> {
    Bound(mpsc::Receiver<T>),
    UnBound(mpsc::UnboundedReceiver<T>),
}

impl<T> std::fmt::Debug for MpscReceiver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bound(_) => write!(f, "MpscReceiver::Bound(<receiver>)"),
            Self::UnBound(_) => write!(f, "MpscReceiver::UnBound(<receiver>)"),
        }
    }
}

impl<T> MpscReceiver<T> {
    pub fn status(&self) -> Status {
        if self.is_closed() && self.is_empty() {
            Status::Closed
        } else if self.is_closed() && !self.is_empty() {
            Status::Draining
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

impl<T> From<mpsc::Receiver<T>> for MpscReceiver<T> {
    fn from(value: mpsc::Receiver<T>) -> Self {
        Self::Bound(value)
    }
}

impl<T> From<mpsc::UnboundedReceiver<T>> for MpscReceiver<T> {
    fn from(value: mpsc::UnboundedReceiver<T>) -> Self {
        Self::UnBound(value)
    }
}
