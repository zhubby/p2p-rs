use std::{env, error::Error};

use libp2p::{
    futures::StreamExt,
    identity,
    ping::{Ping, PingConfig},
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm,
};

use tracing_subscriber;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run().await
}

async fn run() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "INFO");
    tracing_subscriber::fmt::init();
    // 生成密钥对
    let key_pair = identity::Keypair::generate_ed25519();
    // 基于密钥对的公钥，生成节点唯一标识peerId
    let peer_id = PeerId::from(key_pair.public());

    info!("节点ID: {peer_id}");

    // 声明Ping网络行为
    // 网络行为Behaviour：传输(transport)定义如何在网络中发送字节流
    let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

    // 传输
    let transport = libp2p::development_transport(key_pair).await?;

    // 网络管理模块
    // 网络管理模块Swarm：用于管理节点之间的所有活跃连接和挂起连接，并管理所有已打开的子流状态。Swarm是通过传输、网络行为和节点PeerId来创建。
    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    // 在节点随机开启一个端口监听
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // 从命令行参数获取远程节点地址，进行链接。
    if let Some(remote_peer) = std::env::args().nth(1) {
        let remote_peer_multiaddr: Multiaddr = remote_peer.parse()?;
        swarm.dial(remote_peer_multiaddr)?;
        info!("连接远程节点: {remote_peer}");
    }

    loop {
        // 匹配网络事件
        match swarm.select_next_some().await {
            // 监听事件
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("本地监听地址: {address}");
            }
            // 网络行为事件
            SwarmEvent::Behaviour(event) => {
                info!("{:?}", event)
            }

            SwarmEvent::ConnectionClosed {
                peer_id,
                endpoint,
                num_established,
                cause,
            } => {
                info!("退出网络: {:?}，原因: {:?}", peer_id, cause)
            }

            _ => {}
        }
    }
}
