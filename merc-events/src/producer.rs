use std::collections::HashMap;

use lapin::{
    Channel, Connection, ConnectionProperties, Queue, options::QueueDeclareOptions,
    types::FieldTable,
};
use merc_error::Result;

pub struct Producer {
    conn: Connection,
    channel: Channel,
    queues: HashMap<String, Queue>,
}

impl Producer {
    pub async fn connect(uri: &str) -> Result<Self> {
        let conn = Connection::connect(uri, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        let _ = channel
            .queue_declare(
                "memory.create",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            conn,
            channel,
            queues: HashMap::new(),
        })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    pub fn queue(&self, key: &str) -> Option<&Queue> {
        self.queues.get(key)
    }
}
