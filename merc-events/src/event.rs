#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Event<TBody> {
    pub id: uuid::Uuid,
    pub subject: String,
    pub action: String,
    pub body: TBody,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<TBody> Event<TBody> {
    pub fn new() -> EventBuilder<TBody> {
        EventBuilder::new()
    }
}

#[derive(Clone)]
pub struct EventBuilder<TBody> {
    subject: Option<String>,
    action: Option<String>,
    body: Option<TBody>,
}

impl<TBody> EventBuilder<TBody> {
    pub fn new() -> Self {
        Self {
            subject: None,
            action: None,
            body: None,
        }
    }

    pub fn subject<TSubject: ToString>(mut self, subject: TSubject) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    pub fn action<TAction: ToString>(mut self, action: TAction) -> Self {
        self.action = Some(action.to_string());
        self
    }

    pub fn body(mut self, body: TBody) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Event<TBody> {
        Event {
            id: uuid::Uuid::new_v4(),
            subject: self.subject.expect("subject is required"),
            action: self.action.expect("action is required"),
            body: self.body.expect("body is required"),
            created_at: chrono::Utc::now(),
        }
    }
}
