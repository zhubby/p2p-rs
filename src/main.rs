mod chat;

use std::env;
use std::error::Error;
use anyhow::Result;
use chat::ChatBehaviour;
use futures::StreamExt;
use libp2p::{
    development_transport,
    core::upgrade,
    ping::{Ping,PingConfig},
    floodsub, identity, noise,
    swarm::{SwarmBuilder, SwarmEvent, NetworkBehaviour},
    tcp::GenTcpConfig,
    tcp::TokioTcpTransport,
    yamux, Multiaddr, PeerId, Transport,
};
use tokio::io;
use tokio::io::AsyncBufReadExt;

#[macro_use]
extern crate tracing;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "INFO");
    tracing_subscriber::fmt::init();
    // 生成密钥对
    let id_keys = identity::Keypair::generate_ed25519();

    // 基于密钥对的公钥，生成节点唯一标识peerId
    let peer_id = PeerId::from(id_keys.public());
    info!("节点ID: {peer_id}");

    // 创建noise密钥对
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new().into_authentic(&id_keys)?;


    // 创建一个基于tokio的TCP传输层，使用noise进行身份验证。
    // 由于多了一层加密，所以使用yamux基于TCP流进行多路复用。
    let transport = TokioTcpTransport::new(GenTcpConfig::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(yamux::YamuxConfig::default())
        .boxed();

    // 创建 Floodsub 主题
    let floodsub_topic = floodsub::Topic::new("chat");

    // let mut ping_swarm = {
    //     SwarmBuilder::new(development_transport(id_keys).await?, Ping::new(PingConfig::new().with_keep_alive(true)), peer_id).build()
    // };

    // 创建Swarm来管理节点网络及事件。
    let mut swarm = {
        let mut behaviour = ChatBehaviour::new(peer_id).await?;
        // 订阅floodsub topic
        behaviour.floodsub.subscribe(floodsub_topic.clone());

        SwarmBuilder::new(transport, behaviour, peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };

    // 指定一个远程节点，进行手动链接。
    if let Some(to_dial) = std::env::args().nth(1) {
        let addr: Multiaddr = to_dial.parse()?;
        swarm.dial(addr)?;
        info!("链接远程节点: {to_dial}");
    }

    // 从标准输入中读取消息
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // 监听操作系统分配的端口
    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;
    let mut interval = tokio::time::interval(std::time::Duration::new(5,0));

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                // 从标准输入中读取消息后，发布到订阅了floodsub topic的节点上。
                swarm.behaviour_mut().floodsub.publish(floodsub_topic.clone(), line.as_bytes());
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    info!("本地监听地址: {address}");
                }
            }
            // _ = ping_swarm.select_next_some() => {

            // }

            _ = interval.tick() => {
                info!("send message");
                swarm.behaviour_mut().floodsub.publish(floodsub_topic.clone(), "hello");
            }
        }
    }
}
