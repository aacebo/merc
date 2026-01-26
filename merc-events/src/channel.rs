use std::collections::HashMap;

use lapin::{Channel, Connection, ConnectionProperties, options, types};
use merc_error::{Error, Result};

use crate::{Consumer, Key, Producer};

pub struct ChannelConnection {
    conn: Connection,
    channel: Channel,
    queues: HashMap<Key, lapin::Queue>,
}

impl ChannelConnection {
    pub async fn connect(uri: &str) -> Result<Self> {
        let conn = Connection::connect(uri, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;

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

    pub fn queue(&self, key: Key) -> Option<&lapin::Queue> {
        self.queues.get(&key)
    }

    pub async fn consume(self, key: Key) -> Result<Consumer> {
        if !self.queues.contains_key(&key) {
            return Err(Error::builder().message("queue not found").build());
        }

        Consumer::connect(self, key.queue()).await
    }

    pub fn produce(self) -> Producer {
        Producer::connect(self)
    }
}

pub struct ChannelConnector {
    conn: Connection,
    channel: Channel,
    queues: HashMap<Key, lapin::Queue>,
}

impl ChannelConnector {
    pub async fn connect(uri: &str) -> Result<Self> {
        let conn = Connection::connect(uri, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;

        Ok(Self {
            conn,
            channel,
            queues: HashMap::new(),
        })
    }

    pub async fn with_queue(mut self, key: Key) -> Result<Self> {
        self.channel
            .exchange_declare(
                key.exchange(),
                lapin::ExchangeKind::Topic,
                options::ExchangeDeclareOptions::default(),
                types::FieldTable::default(),
            )
            .await?;

        let queue = self
            .channel
            .queue_declare(
                key.queue(),
                options::QueueDeclareOptions::default(),
                types::FieldTable::default(),
            )
            .await?;

        self.channel
            .queue_bind(
                key.queue(),
                key.exchange(),
                &key.to_string(),
                options::QueueBindOptions::default(),
                types::FieldTable::default(),
            )
            .await?;

        self.queues.insert(key, queue);
        Ok(self)
    }

    pub fn build(self) -> ChannelConnection {
        ChannelConnection {
            conn: self.conn,
            channel: self.channel,
            queues: self.queues,
        }
    }
}
