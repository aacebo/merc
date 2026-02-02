use crate::{
    chan::{AsyncSender, Sender},
    tasks::TaskResult,
};

use super::{TaskError, TaskId};

///
/// ## TaskResolver
/// a mutable reference to a Task with helpers to
/// send a result or error
///
pub struct TaskResolver<T: Send + 'static> {
    id: TaskId,
    sender: Option<Box<dyn Sender<Item = TaskResult<T>>>>,
}

impl<T: Send + 'static> TaskResolver<T> {
    pub fn new<S: Sender<Item = TaskResult<T>>>(id: TaskId, sender: S) -> Self {
        Self {
            id,
            sender: Some(Box::new(sender)),
        }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn ok(mut self, value: T) -> Result<(), TaskError> {
        let sender = self.sender.take().expect("sender already consumed");
        sender.send(TaskResult::Ok(value)).map_err(TaskError::from)
    }

    pub fn error<Err: ToString>(mut self, error: Err) -> Result<(), TaskError> {
        let sender = self.sender.take().expect("sender already consumed");
        sender
            .send(TaskResult::Error(TaskError::Custom(error.to_string())))
            .map_err(TaskError::from)
    }

    pub fn cancel(mut self) -> Result<(), TaskError> {
        let sender = self.sender.take().expect("sender already consumed");
        sender.send(TaskResult::Cancelled).map_err(TaskError::from)
    }
}

impl<T: Send + 'static> TaskResolver<T> {
    pub async fn ok_async(mut self, value: T) -> Result<(), TaskError> {
        let sender = self.sender.take().expect("sender already consumed");
        sender
            .send_async(TaskResult::Ok(value))
            .await
            .map_err(TaskError::from)
    }

    pub async fn error_async<Err: ToString>(mut self, error: Err) -> Result<(), TaskError> {
        let sender = self.sender.take().expect("sender already consumed");
        sender
            .send_async(TaskResult::Error(TaskError::Custom(error.to_string())))
            .await
            .map_err(TaskError::from)
    }

    pub async fn cancel_async(mut self) -> Result<(), TaskError> {
        let sender = self.sender.take().expect("sender already consumed");
        sender
            .send_async(TaskResult::Cancelled)
            .await
            .map_err(TaskError::from)
    }
}

impl<T: Send + 'static> Drop for TaskResolver<T> {
    fn drop(&mut self) {
        // Just drop the sender - this closes the channel
        // The receiver will get a closed channel error
        drop(self.sender.take());
    }
}
