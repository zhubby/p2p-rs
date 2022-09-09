mod store;

use axum::{response::Json, routing::get, Router};
use futures::StreamExt;
use libp2p::{
    identity,
    kad::{record::Key, store::MemoryStore, Kademlia, Quorum, Record},
    swarm::{SwarmBuilder, SwarmEvent},
    PeerId, Swarm,
};
use std::net::SocketAddr;
use std::{env, error::Error};
use store::MyBehaviour;
use tokio::io;
use tokio::io::AsyncBufReadExt;

use tracing_subscriber;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run().await
}

async fn run() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "DEBUG");
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
    let app = Router::new().route("/", get(get_value));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let sm : &'static mut Swarm<MyBehaviour> = swarm.take_while(f)
    tokio::spawn(listen_addr(&'static mut swarm));
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
    // 监听操作系统分配的端口
    

    // loop {
    //     tokio::select! {
    //         line = stdin.next_line() => {
    //             let line = line?.expect("stdin closed");
    //             handle_input_line(&mut swarm.behaviour_mut().kademlia, line);
    //         },
    //         event = swarm.select_next_some() => {
    //             if let SwarmEvent::NewListenAddr { address, .. } = event {
    //                 info!("本地监听地址: {address}");
    //             }
    //         }
    //     }
    // }
}

async fn listen_addr<'a>(swarm : &mut Swarm<MyBehaviour>) {
     loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    info!("本地监听地址: {address}");
                }
            }
        }
    }
}

// 处理输入命令
fn handle_input_value(kademlia: &mut Kademlia<MemoryStore>, value: String) {
    let mut args = value.split(' ');

    match args.next() {
        // 处理 GET 命令，获取存储的kv记录
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        error!("Expected key");
                        return;
                    }
                }
            };
            // 获取v记录
            kademlia.get_record(key, Quorum::One);
        }
        // 处理 GET_PROVIDERS 命令，获取存储kv记录的节点PeerId
        Some("GET_PROVIDERS") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        error!("Expected key");
                        return;
                    }
                }
            };
            // 获取存储kv记录的节点
            kademlia.get_providers(key);
        }
        // 处理 PUT 命令，存储kv记录
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            // 将值转换成Vec<u8>类型
            let value = {
                match args.next() {
                    Some(value) => value.as_bytes().to_vec(),
                    None => {
                        error!("Expected value");
                        return;
                    }
                }
            };
            let record = Record {
                key,
                value,
                publisher: None,
                expires: None,
            };
            // 存储kv记录
            kademlia
                .put_record(record, Quorum::One)
                .expect("Failed to store record locally.");
        }
        // 处理 PUT_PROVIDER 命令，保存kv记录的提供者(节点)
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        error!("Expected key");
                        return;
                    }
                }
            };

            kademlia
                .start_providing(key)
                .expect("Failed to start providing key");
        }
        _ => {
            error!("expected GET, GET_PROVIDERS, PUT or PUT_PROVIDER");
        }
    }
}

async fn get_value() -> Json<String> {
    Json("<h1>Hello, World!</h1>".to_string())
}

async fn set_value() -> Json<String> {
    Json("<h1>Hello, World!</h1>".to_string())
}
