// 自定义网络行为，组合Kademlia和mDNS.
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
struct MyBehaviour {
    kademlia: Kademlia<MemoryStore>, // 内存存储
    mdns: Mdns,
}

impl MyBehaviour {
    // 传入peerId，构建MyBehaviour
    async fn new(peer_id: PeerId) -> Result<Self> {
        let store = MemoryStore::new(peer_id);
        let kademlia = Kademlia::new(peer_id, store);

        Ok(Self {
            // floodsub协议初始化
            kademlia,
            // mDNS协议初始化
            mdns: Mdns::new(Default::default()).await?,
        })
    }
}

// 处理mDNS网络行为事件
impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
    // 当产生一个mDNS事件时，该方法被调用。
    fn inject_event(&mut self, event: MdnsEvent) {
        // 发现新节点时，将节点加入到Kademlia网络中。
        if let MdnsEvent::Discovered(list) = event {
            for (peer_id, multiaddr) in list {
                self.kademlia.add_address(&peer_id, multiaddr);
            }
        }
    }
}

// 处理Kademlia网络行为事件
impl NetworkBehaviourEventProcess<KademliaEvent> for MyBehaviour {
    // 当产生一个kademlia事件时，该方法被调用。
    fn inject_event(&mut self, message: KademliaEvent) {
        if let KademliaEvent::OutboundQueryCompleted { result, .. } = message {
            match result {
                // 查询提供key的节点事件
                QueryResult::GetProviders(Ok(ok)) => {
                    for peer in ok.providers {
                        println!(
                            "节点 {:?} 提供了key {:?}",
                            peer,
                            std::str::from_utf8(ok.key.as_ref()).unwrap()
                        );
                    }
                }
                QueryResult::GetProviders(Err(err)) => {
                    eprintln!("Failed to get providers: {:?}", err);
                }
                // 查询存储记录事件
                QueryResult::GetRecord(Ok(ok)) => {
                    for PeerRecord {
                        record: Record { key, value, .. },
                        ..
                    } in ok.records
                    {
                        println!(
                            "获取存储记录 {:?} {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );
                    }
                }
                QueryResult::GetRecord(Err(err)) => {
                    eprintln!("Failed to get record: {:?}", err);
                }
                // 记录存储成功事件
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    println!(
                        "成功存储记录  {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::PutRecord(Err(err)) => {
                    eprintln!("Failed to put record: {:?}", err);
                }
                // 成功存储记录提供者事件
                QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                    println!(
                        "成功存储记录提供者 {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::StartProviding(Err(err)) => {
                    eprintln!("Failed to put provider record: {:?}", err);
                }
                _ => {}
            }
        }
    }
}