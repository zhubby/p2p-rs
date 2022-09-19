use anyhow::Result;
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent},
    mdns::{Mdns, MdnsEvent, MdnsConfig},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour, PeerId,
};

// 自定义网络行为，组合floodsub和mDNS。
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct ChatBehaviour {
    // 小型网络广播协议
    pub floodsub: Floodsub,
    // 本地节点发现
    pub mdns: Mdns,
}

impl ChatBehaviour {
    // 传入peerId，构建MyBehaviour
    pub async fn new(id: PeerId) -> Result<Self> {
        // let mdns_config = MdnsConfig { ttl: todo!(), query_interval: todo!(), enable_ipv6: todo!() };
        Ok(Self {
            // floodsub协议初始化
            floodsub: Floodsub::new(id),

           
            // mDNS协议初始化
            mdns: Mdns::new(Default::default()).await?,
        })
    }
}

// 处理Floodsub网络行为事件
// #[derive(NetworkBehaviour)] 选择广播事件实现NetworkBehaviourEventProcess接口
impl NetworkBehaviourEventProcess<FloodsubEvent> for ChatBehaviour {
    // 当产生一个floodsub事件时，该方法被调用。
    fn inject_event(&mut self, message: FloodsubEvent) {
        // 显示接收到的消息及来源
        if let FloodsubEvent::Message(message) = message {
            info!(
                "收到消息: '{:?}' 来自 {:?}",
                String::from_utf8_lossy(&message.data),
                message.source,
            );
        }
    }
}

// 处理mDNS网络行为事件
impl NetworkBehaviourEventProcess<MdnsEvent> for ChatBehaviour {
    // 当产生一个mDNS事件时，该方法被调用。
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            // 发现新节点时，将节点添加到传播消息的节点列表中。
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.floodsub.add_node_to_partial_view(peer);
                    info!("在网络中加入节点: {peer} ");
                }
            }
            // 当节点失效时，从传播消息的节点列表中删除一个节点。
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                        info!("从网络中删除节点: {peer} ");
                    }
                }
            }
        }
    }
}
