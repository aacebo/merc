mod channel;
mod consumer;
mod event;
mod key;
mod producer;

pub use channel::*;
pub use consumer::*;
pub use event::*;
pub use key::*;
pub use producer::*;

pub async fn connect(uri: &str) -> merc_error::Result<ChannelConnector> {
    ChannelConnector::connect(uri).await
}
