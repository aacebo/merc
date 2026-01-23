use crate::entity::{Status, Trace};

#[derive(Debug, Clone)]
pub struct TraceBuilder {
    parent_id: Option<uuid::Uuid>,
    request_id: Option<String>,
    status: Status,
    status_message: Option<String>,
}

impl TraceBuilder {
    pub fn new() -> Self {
        Self {
            parent_id: None,
            request_id: None,
            status: Status::Ok,
            status_message: None,
        }
    }

    pub fn parent_id(mut self, parent_id: uuid::Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn status_message(mut self, message: impl Into<String>) -> Self {
        self.status_message = Some(message.into());
        self
    }

    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.status = Status::Error;
        self.status_message = Some(message.into());
        self
    }

    pub fn build(self) -> Trace {
        Trace {
            id: uuid::Uuid::new_v4(),
            parent_id: self.parent_id,
            request_id: self.request_id,
            status: self.status,
            status_message: self.status_message,
            started_at: chrono::Utc::now(),
            ended_at: None,
        }
    }
}

impl Default for TraceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
