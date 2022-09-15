mod store;

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, Path},
    http::StatusCode,
    response::Json,
    routing::{get,post},
    Router,
};
use futures::StreamExt;
use libp2p::{
    identity,
    kad::{record::Key, Quorum, Record},
    swarm::{SwarmBuilder, SwarmEvent},
    PeerId,
};
use std::{collections::HashMap, net::SocketAddr};
use std::{env, error::Error};
use store::MyBehaviour;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;
use tracing_subscriber;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run().await
}

async fn run() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    tracing_subscriber::fmt::init();
    // 生成密钥对
    let key_pair = identity::Keypair::generate_ed25519();

    // 基于密钥对的公钥，生成节点唯一标识peerId
    let peer_id = PeerId::from(key_pair.public());
    info!("节点ID: {peer_id}");

    // 在Mplex协议上建立一个加密的，启用dns的TCP传输
    let transport = libp2p::development_transport(key_pair).await?;

    // 创建Swarm网络管理器，来管理节点网络及事件。
    let mut swarm = {
        let behaviour = MyBehaviour::new(peer_id).await?;

        SwarmBuilder::new(transport, behaviour, peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };
    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

    // 从标准输入中读取消息
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    // let &mut kademlia = &mut swarm.behaviour_mut().kademlia;
    let mut channel = StorageChannel::new();
    let app = Router::new()
        .route("/", post(set_value))
        .route("/:key", get(get_value))
        .route("/providers/:key", get(get_providers).post(set_provider))
        .layer(Extension(channel.sender));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tokio::spawn(async move {
        loop {
            tokio::select! {
                event = swarm.select_next_some() => {
                    if let SwarmEvent::NewListenAddr { address, .. } = event {
                        info!("P2P网络本地监听地址: {address}");
                    }
                }

                recv = channel.receiver.recv() => {
                    if let Some(msg) = recv  {
                        info!("receive from channel {:#?}",&msg);
                        match msg {
                            Message::Set(k,v) => {
                                let record = Record {
                                    key: Key::new(&k),
                                    value: v.as_bytes().to_vec(),
                                    publisher: None,
                                    expires: None,
                                };

                                swarm.behaviour_mut().kademlia.put_record(record, Quorum::One).expect("Failed to store record locally.");
                            },

                            Message::SetProvider(k) => {
                                swarm.behaviour_mut().kademlia.start_providing(Key::new(&k)).expect("Failed to start providing key.");
                            },

                            Message::Get(k) => {
                                swarm.behaviour_mut().kademlia.get_record(Key::new(&k), Quorum::One);
                            },

                            Message::GetProvider(k) => {
                                swarm.behaviour_mut().kademlia.get_providers(Key::new(&k));
                            }
                        }
                    }
                }
            }
        }
    });
    info!("web服务监听地址：{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[derive(Debug)]
enum Message {
    Set(String, String),
    Get(String),
    SetProvider(String),
    GetProvider(String),
}

struct StorageChannel {
    sender: mpsc::UnboundedSender<Message>,
    receiver: mpsc::UnboundedReceiver<Message>,
}

struct MessageSender(mpsc::UnboundedSender<Message>);

#[async_trait]
impl<B> FromRequest<B> for MessageSender
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(sender) = Extension::<mpsc::UnboundedSender<Message>>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(sender))
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

impl StorageChannel {
    pub fn new() -> Self {
        let (s, r) = mpsc::unbounded_channel::<Message>();
        Self {
            sender: s,
            receiver: r,
        }
    }
}

async fn get_value(MessageSender(sender): MessageSender,Path(key) : Path<String>) -> Json<String> {
    sender.send(Message::Get(key));
    Json("<h1>Hello, World!</h1>".to_string())
}

async fn set_value(
    MessageSender(sender): MessageSender,
    req: Json<HashMap<String, String>>,
) -> Json<String> {
    for (k, v) in req.iter() {
        sender.send(Message::Set(k.clone(), v.clone()));
    }
    Json("<h1>Hello, World!</h1>".to_string())
}

async fn set_provider(MessageSender(sender): MessageSender,Path(key) : Path<String>) -> Json<String> {
    sender.send(Message::SetProvider(key));
    Json("<h1>Hello, World!</h1>".to_string())
}

async fn get_providers(MessageSender(sender): MessageSender,Path(key) : Path<String>) -> Json<String> {
    sender.send(Message::GetProvider(key));
    Json("<h1>Hello, World!</h1>".to_string())
}
