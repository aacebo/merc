use crate::entity::{Action, Target, TraceAction};

#[derive(Debug, Clone)]
pub struct TraceActionBuilder {
    trace_id: uuid::Uuid,
    target_id: uuid::Uuid,
    target: Target,
    action: Action,
}

impl TraceActionBuilder {
    pub fn new(
        trace_id: uuid::Uuid,
        target_id: uuid::Uuid,
        target: Target,
        action: Action,
    ) -> Self {
        Self {
            trace_id,
            target_id,
            target,
            action,
        }
    }

    pub fn build(self) -> TraceAction {
        TraceAction {
            trace_id: self.trace_id,
            target_id: self.target_id,
            target: self.target,
            action: self.action,
            created_at: chrono::Utc::now(),
        }
    }
}
