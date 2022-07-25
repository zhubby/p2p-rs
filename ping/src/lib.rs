use std::error::Error;

use libp2p::{
    futures::StreamExt,
    identity,
    ping::{Ping, PingConfig},
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm,
};

pub async fn run() -> Result<(),Box<dyn Error>> {
    let key_pair = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key_pair.public());
    println!("节点ID: {peer_id}");
    let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));
    let transport = libp2p::development_transport(key_pair).await?;
    let mut swarm = Swarm::new(transport, behaviour, peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    
    if let Some(remote_peer) = std::env::args().nth(1) {
        let remote_peer_multiaddr: Multiaddr = remote_peer.parse()?;
        swarm.dial(remote_peer_multiaddr)?;
        println!("链接远程节点: {remote_peer}");
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("本地监听地址: {address}");
            }
            SwarmEvent::Behaviour(event) => println!("{:?}", event),
            _ => {}
        }
    }
}
