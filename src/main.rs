use std::error::Error;

#[cfg(feature = "ping")]
use ping;
#[cfg(feature = "chatroom")]
use chatroom;
#[cfg(feature = "kv")]
use distributed_kv_store as dkv;
#[cfg(feature = "dfs")]
use distributed_fs as dfs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "ping")]
    ping::run().await;
    #[cfg(feature = "chatroom")]
    chatroom::run().await;
    #[cfg(feature = "dkv")]
    dkv::run().await;
    #[cfg(feature = "dfs")]
    dfs::run().await;
    Ok(())
}
