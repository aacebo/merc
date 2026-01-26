mod channel;
mod config;
mod consumer;
mod event;
mod options;
mod producer;

pub use channel::*;
pub use config::*;
pub use consumer::*;
pub use event::*;
pub use options::*;
pub use producer::*;

pub async fn connect(uri: &str) -> merc_error::Result<ChannelConnection> {
    ChannelConnection::connect(uri).await
}
