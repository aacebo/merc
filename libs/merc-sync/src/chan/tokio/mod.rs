mod receiver;
mod sender;

pub use receiver::*;
pub use sender::*;

use tokio::sync::mpsc;

#[track_caller]
pub fn open<T: std::fmt::Debug + Send + 'static>() -> (TokioSender<T>, TokioReceiver<T>) {
    let (sender, receiver) = mpsc::unbounded_channel();

    (
        TokioSender::new(MpscSender::from(sender)),
        TokioReceiver::new(MpscReceiver::from(receiver)),
    )
}

#[track_caller]
pub fn alloc<T: std::fmt::Debug + Send + 'static>(
    capacity: usize,
) -> (TokioSender<T>, TokioReceiver<T>) {
    let (sender, receiver) = mpsc::channel(capacity);

    (
        TokioSender::new(MpscSender::from(sender)),
        TokioReceiver::new(MpscReceiver::from(receiver)),
    )
}
