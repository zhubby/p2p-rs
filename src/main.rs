use std::error::Error;
use ping;
use chatroom;
use distributed_kv_store as kv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[feature(enable = "ping")]
    ping::run().await;
    #[feature(enable = "chatroom")]
    chatroom::run().await;
    #[feature(enable = "kv")]
    kv::run().await;
    Ok(())
}
