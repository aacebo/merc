use lapin::{options, protocol};
use merc_error::Result;

use crate::{ChannelConnection, Event};

pub struct Producer {
    conn: ChannelConnection,
}

impl Producer {
    pub fn connect(conn: ChannelConnection) -> Self {
        Self { conn }
    }

    pub fn conn(&self) -> &ChannelConnection {
        &self.conn
    }

    pub async fn enqueue<TBody: serde::Serialize>(&self, event: Event<TBody>) -> Result<()> {
        let payload = serde_json::to_vec(&event)?;
        let _ = self
            .conn
            .channel()
            .basic_publish(
                event.key.exchange(),
                &event.key.to_string(),
                options::BasicPublishOptions::default(),
                &payload,
                protocol::basic::AMQPProperties::default(),
            )
            .await?;

        Ok(())
    }
}
