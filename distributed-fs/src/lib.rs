mod behaviour;
mod client;
mod event;
mod protocol;

use behaviour::ComposedBehaviour;
use clap::Parser;
use client::Client;
use event::{Event, EventLoop};
use futures::FutureExt;
use libp2p::Multiaddr;
use libp2p::{
    identity::{self, ed25519},
    kad::{store::MemoryStore, Kademlia},
    multiaddr::Protocol,
    request_response::{ProtocolSupport, RequestResponse},
    swarm::SwarmBuilder,
    PeerId,
};
use protocol::*;
use std::env;
use std::{error::Error, iter, path::PathBuf};
use tokio::sync::mpsc::{self, Receiver};
use tracing_subscriber;

#[macro_use]
extern crate tracing;

#[derive(Debug, Parser)]
#[clap(name = "dfs")]
pub struct Opt {
    // 生成密钥对的种子
    #[clap(long)]
    pub secret_key_seed: Option<u8>,

    // 节点地址
    #[clap(long)]
    pub peer: Option<Multiaddr>,

    // 监听地址
    #[clap(long)]
    pub listen_address: Option<Multiaddr>,

    // 子命令
    #[clap(subcommand)]
    pub argument: CliArgument,
}

#[derive(Debug, Parser)]
pub enum CliArgument {
    // 提供文件子命令
    Provide {
        #[clap(long)]
        path: PathBuf, // 文件全路径
        #[clap(long)]
        name: String, // 文件名称
    },
    // 获取文件内容子命令
    Get {
        #[clap(long)]
        name: String, // 文件名称
    },
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "DEBUG");
    tracing_subscriber::fmt::init();
    let opt = Opt::parse();

    let (network_client, network_events, network_event_loop) =
        network(opt.secret_key_seed).await?;

    tokio::spawn(async move {
        network_event_loop.run().await;
    });

    process_args(opt, network_client, network_events).await?;

    Ok(())
}

async fn process_args(
    opt: Opt,
    mut network_client: Client,
    mut network_events: Receiver<Event>,
) -> Result<(), Box<dyn Error>> {
    match opt.listen_address {
        Some(addr) => network_client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),
        None => network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    if let Some(addr) = opt.peer {
        let peer_id = match addr.iter().last() {
            Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash).expect("Valid hash."),
            _ => return Err("Expect peer multiaddr to contain peer ID.".into()),
        };
        network_client
            .dial(peer_id, addr)
            .await
            .expect("Dial to succeed");
    }

    match opt.argument {
        CliArgument::Provide { path, name } => {
            // Advertise oneself as a provider of the file on the DHT.
            network_client.start_providing(name.clone()).await;

            loop {
                match network_events.recv().await {
                    // Reply with the content of the file on incoming requests.
                    Some(Event::InboundRequest { request, channel }) => {
                        if request == name {
                            let file_content = std::fs::read_to_string(&path)?;
                            network_client.respond_file(file_content, channel).await;
                        }
                    }
                    e => todo!("{:?}", e),
                }
            }
        }

        CliArgument::Get { name } => {
            // 找到提供该文件的所有节点
            let providers = network_client.get_providers(name.clone()).await;
            if providers.is_empty() {
                return Err(format!("Could not find provider for file {}.", name).into());
            }

            // 从每个节点请求文件的内容。
            let requests = providers.into_iter().map(|p| {
                let mut network_client = network_client.clone();
                let name = name.clone();
                async move { network_client.request_file(p, name).await }.boxed()
            });

            // 等待请求，一旦有一个请求成功，就忽略剩下的请求。
            let file = futures::future::select_ok(requests)
                .await
                .map_err(|_| "None of the providers returned file.")?
                .0;

            println!("Content of file {}: {}", name, file);
        }
    }

    Ok(())
}

pub async fn network(
    secret_key_seed: Option<u8>,
) -> Result<(Client, Receiver<Event>, EventLoop), Box<dyn Error>> {
    // 创建密钥对
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            let secret_key = ed25519::SecretKey::from_bytes(&mut bytes).expect(
                "this returns `Err` only if the length is wrong; the length is correct; qed",
            );
            identity::Keypair::Ed25519(secret_key.into())
        }
        None => identity::Keypair::generate_ed25519(),
    };
    // 根据公钥生成节点ID
    let peer_id = id_keys.public().to_peer_id();

    // 构建网络层管理组件Swarm
    let swarm = SwarmBuilder::new(
        libp2p::development_transport(id_keys).await?,
        ComposedBehaviour {
            kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
            request_response: RequestResponse::new(
                FileSwapCodec(),
                iter::once((FileSwapProtocol(), ProtocolSupport::Full)),
                Default::default(),
            ),
        },
        peer_id,
    )
    .build();

    let (command_sender, command_receiver) = mpsc::channel(1);
    let (event_sender, event_receiver) = mpsc::channel(1);

    Ok((
        Client::new(command_sender),
        event_receiver,
        EventLoop::new(swarm, command_receiver, event_sender),
    ))
}
