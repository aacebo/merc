use tokio::sync::mpsc;

use crate::chan::{Channel, Receiver, Status};

pub enum TokioReceiver<T> {
    Bound(Status, mpsc::Receiver<T>),
    UnBound(Status, mpsc::UnboundedReceiver<T>),
}

impl<T> TokioReceiver<T> {
    pub fn is_bound(&self) -> bool {
        match self {
            Self::Bound(_, _) => true,
            _ => false,
        }
    }

    pub fn is_unbound(&self) -> bool {
        match self {
            Self::UnBound(_, _) => true,
            _ => false,
        }
    }
}

impl<T> From<mpsc::Receiver<T>> for TokioReceiver<T> {
    fn from(value: mpsc::Receiver<T>) -> Self {
        Self::Bound(
            Status::bound(value.max_capacity()).with_len(value.len()),
            value,
        )
    }
}

impl<T> From<mpsc::UnboundedReceiver<T>> for TokioReceiver<T> {
    fn from(value: mpsc::UnboundedReceiver<T>) -> Self {
        Self::UnBound(Status::default().with_len(value.len()), value)
    }
}

impl<T> Channel for TokioReceiver<T> {
    fn status(&self) -> Status {
        match self {
            Self::Bound(s, _) => *s,
            Self::UnBound(s, _) => *s,
        }
    }
}

impl<T: Send + 'static> Receiver for TokioReceiver<T> {
    type Item = T;

    fn recv(&self) -> Result<Self::Item, crate::chan::error::RecvError> {
        todo!()
    }
}
