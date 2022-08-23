use std::{collections::HashSet, error::Error};

use libp2p::{request_response::ResponseChannel, Multiaddr, PeerId};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use crate::protocol::FileResponse;

#[derive(Debug)]
pub enum Command {
    // 监听本地端口命令
    StartListening {
        // 本地监听地址
        addr: Multiaddr,
        // 用于发送命令执行状态的通道
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    // 链接给定节点命令
    Dial {
        // 节点ID
        peer_id: PeerId,
        // 节点地址
        peer_addr: Multiaddr,
        // 用于发送命令执行状态的通道
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    // 宣称本节点提供共享文件命令
    StartProviding {
        // 文件名称
        file_name: String,
        // 用于发送命令执行状态的通道
        sender: oneshot::Sender<()>,
    },
    // 获取提供共享文件的节点命令
    GetProviders {
        // 文件名称
        file_name: String,
        // 用于发送命令执行状态的通道
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    // 请求共享文件命令
    RequestFile {
        // 文件名称
        file_name: String,
        // 节点ID
        peer: PeerId,
        // 用于发送命令执行状态的通道
        sender: oneshot::Sender<Result<String, Box<dyn Error + Send>>>,
    },
    // 返回共享文件内容命令
    RespondFile {
        // 文件名称
        file: String,
        // 返回文件内容
        channel: ResponseChannel<FileResponse>,
    },
}

// 用于发送命令的Client
#[derive(Clone)]
pub struct Client {
    // 将命令发送到mpsc通道
    sender: mpsc::Sender<Command>,
}

impl Client {
    pub fn new(sender: Sender<Command>) -> Client {
        Client { sender }
    }

    pub async fn start_listening(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn start_providing(&mut self, file_name: String) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartProviding { file_name, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }

    pub async fn get_providers(&mut self, file_name: String) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetProviders { file_name, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn request_file(
        &mut self,
        peer: PeerId,
        file_name: String,
    ) -> Result<String, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestFile {
                file_name,
                peer,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    #[allow(dead_code)]
    pub async fn respond_file(&mut self, file: String, channel: ResponseChannel<FileResponse>) {
        self.sender
            .send(Command::RespondFile { file, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }
}