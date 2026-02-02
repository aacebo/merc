use std::{any::type_name_of_val, time::Duration};

use tokio::sync::mpsc;

use crate::chan::{Channel, Sender, Status, error::SendError};

#[derive(Clone)]
pub struct TokioSender<T> {
    sender: MpscSender<T>,
}

impl<T> std::fmt::Debug for TokioSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioSender")
            .field("sender", &self.sender)
            .finish()
    }
}

impl<T> TokioSender<T> {
    pub fn new(sender: MpscSender<T>) -> Self {
        Self { sender }
    }
}

impl<T> std::ops::Deref for TokioSender<T> {
    type Target = MpscSender<T>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl<T> Channel for TokioSender<T> {
    fn status(&self) -> Status {
        self.sender.status()
    }

    fn len(&self) -> usize {
        self.sender.capacity()
    }

    fn capacity(&self) -> Option<usize> {
        self.sender.max_capacity()
    }
}

impl<T: Send + 'static> Sender for TokioSender<T> {
    type Item = T;

    fn send(&self, result: T) -> Result<(), SendError> {
        match self.sender.block_send(result) {
            Err(_) => {
                if self.status().is_closed() {
                    Err(SendError::Closed)
                } else {
                    Err(SendError::Full)
                }
            }
            Ok(_) => Ok(()),
        }
    }
}

#[derive(Clone)]
pub enum MpscSender<T> {
    Bound(mpsc::Sender<T>),
    UnBound(mpsc::UnboundedSender<T>),
}

impl<T> std::fmt::Debug for MpscSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bound(_) => write!(f, "MpscSender::Bound(<sender>)"),
            Self::UnBound(_) => write!(f, "MpscSender::UnBound(<sender>)"),
        }
    }
}

impl<T> MpscSender<T> {
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

    pub async fn reserve(&self) -> Result<mpsc::Permit<'_, T>, mpsc::error::SendError<()>> {
        match self {
            Self::Bound(v) => v.reserve().await,
            v => panic!("attempted use of {}::reserve", type_name_of_val(v)),
        }
    }

    pub async fn send(&self, value: T) -> Result<(), mpsc::error::SendError<T>> {
        match self {
            Self::Bound(v) => v.send(value).await,
            Self::UnBound(v) => v.send(value),
        }
    }

    pub async fn send_timeout(
        &self,
        value: T,
        timeout: Duration,
    ) -> Result<(), mpsc::error::SendTimeoutError<T>> {
        match self {
            Self::Bound(v) => v.send_timeout(value, timeout).await,
            v => panic!("attempted use of {}::send_timeout", type_name_of_val(v)),
        }
    }

    pub fn block_send(&self, value: T) -> Result<(), mpsc::error::SendError<T>> {
        match self {
            Self::Bound(v) => v.blocking_send(value),
            Self::UnBound(v) => v.send(value),
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

impl<T> From<mpsc::Sender<T>> for MpscSender<T> {
    fn from(value: mpsc::Sender<T>) -> Self {
        Self::Bound(value)
    }
}

impl<T> From<mpsc::UnboundedSender<T>> for MpscSender<T> {
    fn from(value: mpsc::UnboundedSender<T>) -> Self {
        Self::UnBound(value)
    }
}

#[derive(Clone)]
pub enum MpscWeakSender<T> {
    Bound(mpsc::WeakSender<T>),
    UnBound(mpsc::WeakUnboundedSender<T>),
}

impl<T> std::fmt::Debug for MpscWeakSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bound(_) => write!(f, "MpscWeakSender::Bound(<weak_sender>)"),
            Self::UnBound(_) => write!(f, "MpscWeakSender::UnBound(<weak_sender>)"),
        }
    }
}

impl<T> MpscWeakSender<T> {
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

impl<T> From<mpsc::WeakSender<T>> for MpscWeakSender<T> {
    fn from(value: mpsc::WeakSender<T>) -> Self {
        Self::Bound(value)
    }
}

impl<T> From<mpsc::WeakUnboundedSender<T>> for MpscWeakSender<T> {
    fn from(value: mpsc::WeakUnboundedSender<T>) -> Self {
        Self::UnBound(value)
    }
}
