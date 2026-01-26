use lapin::{Channel, Connection, ConnectionProperties, options, types};
use merc_error::Result;

use crate::Consumer;

pub struct ChannelConnection {
    conn: Connection,
    channel: Channel,
}

impl ChannelConnection {
    pub async fn connect(uri: &str) -> Result<Self> {
        let conn = Connection::connect(uri, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        Ok(Self { conn, channel })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub async fn consume(self, queue: &str) -> Result<Consumer> {
        let _ = self
            .channel
            .queue_declare(
                queue,
                options::QueueDeclareOptions::default(),
                types::FieldTable::default(),
            )
            .await?;

        Consumer::connect(self, queue).await
    }
}
