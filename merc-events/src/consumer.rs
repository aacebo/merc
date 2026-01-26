use futures_lite::StreamExt;
use lapin::{options::BasicConsumeOptions, types::FieldTable};
use merc_error::Result;

use crate::{ChannelConnection, Event};

pub struct Consumer {
    conn: ChannelConnection,
    consumer: lapin::Consumer,
}

impl Consumer {
    pub async fn connect(conn: ChannelConnection, queue: &str) -> Result<Self> {
        let consumer = conn
            .channel()
            .basic_consume(
                queue,
                "merc[worker]",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Self { conn, consumer })
    }

    pub fn conn(&self) -> &ChannelConnection {
        &self.conn
    }

    pub async fn dequeue<T: for<'a> serde::Deserialize<'a>>(&mut self) -> Option<Result<Event<T>>> {
        let delivery = match self.consumer.next().await? {
            Err(err) => return Some(Err(err.into())),
            Ok(v) => v,
        };

        let string = String::from_utf8_lossy(&delivery.data);
        let data: Event<T> = match serde_json::from_slice(string.as_bytes()) {
            Err(err) => return Some(Err(err.into())),
            Ok(v) => v,
        };

        Some(Ok(data))
    }
}
