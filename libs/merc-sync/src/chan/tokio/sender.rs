use tokio::sync::mpsc;

use crate::chan::{Channel, Sender, Status};

pub enum TokioSender<T> {
    Bound(Status, mpsc::Sender<T>),
    UnBound(Status, mpsc::UnboundedSender<T>),
}

impl<T> TokioSender<T> {
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

impl<T> From<mpsc::Sender<T>> for TokioSender<T> {
    fn from(value: mpsc::Sender<T>) -> Self {
        Self::Bound(
            Status::bound(value.max_capacity()).with_len(value.capacity()),
            value,
        )
    }
}

impl<T> From<mpsc::UnboundedSender<T>> for TokioSender<T> {
    fn from(value: mpsc::UnboundedSender<T>) -> Self {
        Self::UnBound(Status::default().with_len(value.strong_count()), value)
    }
}

impl<T> Channel for TokioSender<T> {
    fn status(&self) -> Status {
        match self {
            Self::Bound(s, _) => *s,
            Self::UnBound(s, _) => *s,
        }
    }
}

impl<T: Send + Sync + 'static> Sender for TokioSender<T> {
    type Item = T;

    fn send(&self, _: Self::Item) -> Result<(), crate::chan::error::SendError> {
        todo!()
    }
}
