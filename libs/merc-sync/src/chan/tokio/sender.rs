use std::any::type_name_of_val;

use tokio::sync::mpsc;

use crate::chan::{Channel, Sender, Status};

#[derive(Debug)]
pub struct TokioSender<T: std::fmt::Debug> {
    status: Status,
    inner: MpscSender<T>,
}

impl<T: std::fmt::Debug> From<MpscSender<T>> for TokioSender<T> {
    fn from(value: MpscSender<T>) -> Self {
        let status = match &value {
            MpscSender::Bound(v) => Status::bound(v.max_capacity()).with_len(v.capacity()),
            _ => Status::default(),
        };

        Self {
            status,
            inner: value,
        }
    }
}

impl<T: std::fmt::Debug> std::ops::Deref for TokioSender<T> {
    type Target = MpscSender<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: std::fmt::Debug> Channel for TokioSender<T> {
    fn status(&self) -> Status {
        self.status
    }
}

impl<T: std::fmt::Debug + Send + 'static> Sender for TokioSender<T> {
    type Item = T;

    fn send(&self, _: Self::Item) -> Result<(), crate::chan::error::SendError> {
        todo!()
    }
}

#[derive(Debug)]
pub enum MpscSender<T: std::fmt::Debug> {
    Bound(mpsc::Sender<T>),
    UnBound(mpsc::UnboundedSender<T>),
}

impl<T: std::fmt::Debug> MpscSender<T> {
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

    pub fn is_closed(&self) -> bool {
        match self {
            Self::Bound(v) => v.is_closed(),
            Self::UnBound(v) => v.is_closed(),
        }
    }

    pub async fn closed(&self) {
        match self {
            Self::Bound(v) => v.closed().await,
            Self::UnBound(v) => v.closed().await,
        }
    }

    pub fn capacity(&self) -> usize {
        match self {
            Self::Bound(v) => v.capacity(),
            v => panic!("attempted use of {}::capacity", type_name_of_val(v)),
        }
    }

    pub fn max_capacity(&self) -> Option<usize> {
        match self {
            Self::Bound(v) => Some(v.max_capacity()),
            _ => None,
        }
    }

    pub fn weak_count(&self) -> usize {
        match self {
            Self::Bound(v) => v.weak_count(),
            Self::UnBound(v) => v.weak_count(),
        }
    }

    pub fn strong_count(&self) -> usize {
        match self {
            Self::Bound(v) => v.strong_count(),
            Self::UnBound(v) => v.strong_count(),
        }
    }

    pub fn downgrade(&self) -> MpscWeakSender<T>
    where
        T: Clone,
    {
        match self {
            Self::Bound(v) => v.downgrade().into(),
            Self::UnBound(v) => v.downgrade().into(),
        }
    }
}

impl<T: Clone + std::fmt::Debug> Clone for MpscSender<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Bound(v) => v.clone().into(),
            Self::UnBound(v) => v.clone().into(),
        }
    }
}

impl<T: std::fmt::Debug> From<mpsc::Sender<T>> for MpscSender<T> {
    fn from(value: mpsc::Sender<T>) -> Self {
        Self::Bound(value)
    }
}

impl<T: std::fmt::Debug> From<mpsc::UnboundedSender<T>> for MpscSender<T> {
    fn from(value: mpsc::UnboundedSender<T>) -> Self {
        Self::UnBound(value)
    }
}

#[derive(Debug)]
pub enum MpscWeakSender<T: std::fmt::Debug> {
    Bound(mpsc::WeakSender<T>),
    UnBound(mpsc::WeakUnboundedSender<T>),
}

impl<T: std::fmt::Debug> MpscWeakSender<T> {
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

    pub fn weak_count(&self) -> usize {
        match self {
            Self::Bound(v) => v.weak_count(),
            Self::UnBound(v) => v.weak_count(),
        }
    }

    pub fn strong_count(&self) -> usize {
        match self {
            Self::Bound(v) => v.strong_count(),
            Self::UnBound(v) => v.strong_count(),
        }
    }

    pub fn upgrade(&self) -> Option<MpscSender<T>> {
        match self {
            Self::Bound(v) => Some(v.upgrade()?.into()),
            Self::UnBound(v) => Some(v.upgrade()?.into()),
        }
    }
}

impl<T: Clone + std::fmt::Debug> Clone for MpscWeakSender<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Bound(v) => v.clone().into(),
            Self::UnBound(v) => v.clone().into(),
        }
    }
}

impl<T: std::fmt::Debug> From<mpsc::WeakSender<T>> for MpscWeakSender<T> {
    fn from(value: mpsc::WeakSender<T>) -> Self {
        Self::Bound(value)
    }
}

impl<T: std::fmt::Debug> From<mpsc::WeakUnboundedSender<T>> for MpscWeakSender<T> {
    fn from(value: mpsc::WeakUnboundedSender<T>) -> Self {
        Self::UnBound(value)
    }
}
