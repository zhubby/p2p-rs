use std::error::Error;
use ping;
use chatroom;
use distributed_kv_store as kv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "ping")]
    ping::run().await;
    #[cfg(feature = "chatroom")]
    chatroom::run().await;
    #[cfg(feature = "kv")]
    kv::run().await;
    Ok(())
}
