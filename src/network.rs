use crate::core::block_chain::BlockChain;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, oneshot};

const LOCAL: &str = "0.0.0.0:4321";

// TODO: add network to chain interactions

// nodes -> network <-> chain { tx_of_addr }
#[derive(Clone, Serialize, Deserialize, Debug)]
enum MsgEvent {
    PushTrx { addr: String, tx_bytes: Vec<u8> }, // 4
    TxsOfAddr { addr: String },                  // 1
    IsKnownAddr { addr: String },                // 2
    RegisterMinner { addr: String },             // 3
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum MsgPropagation {
    Broadcast,
    ToChain,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct NetworkMsg {
    event: MsgEvent,
    propagation: MsgPropagation,
}

type MessageTx = broadcast::Sender<NetworkMsg>;
type MessageRecv = broadcast::Receiver<NetworkMsg>;
struct SocketHandler {
    client_id: u8,
    socket: TcpStream,
    producer: MessageTx,
    consumer: MessageRecv,
    shared_block_chain: Arc<Mutex<BlockChain>>,
}

impl SocketHandler {
    pub fn new(
        client_id: u8,
        socket: TcpStream,
        producer: MessageTx,
        consumer: MessageRecv,
        shared_block_chain: Arc<Mutex<BlockChain>>,
    ) -> Self {
        SocketHandler {
            client_id,
            socket,
            producer,
            consumer,
            shared_block_chain,
        }
    }

    pub async fn process(&mut self) {
        println!("process:cid {}", self.client_id);
        let (mut rd, _) = self.socket.split();
        let mut buf = [0u8; 256];

        loop {
            tokio::select! {
                rd_result = rd.read(&mut buf) => {
                    match rd_result {
                        Err(_) => {
                            println!("process:bytes {}, cid {}", buf.len(), self.client_id);
                            break;
                        }
                        Ok(0) => {
                            println!("process:socker closed || dropped");
                            break;
                        }
                        Ok(_) => {
                            println!("process:read");
                            let msg_result: bincode::Result<NetworkMsg> = bincode::deserialize(&buf[..]);
                            if let Err(e) = msg_result {
                                eprintln!("process:tcp_msg_result:err {:?}", e);
                                break;
                            };
                            self.process_msg(msg_result.unwrap()).await;

                            // Broadcasting
                            // println!("process:{}:tcpmsg:send {:?}", self.client_id, msg);
                            // self.producer.send(t);
                            break;
                        }
                    }
                },
                recv_result = self.consumer.recv() => {
                    println!("process:{}:recv_result {:?}", self.client_id, recv_result);
                }
            }
        }
    }

    pub async fn process_msg(&mut self, msg: NetworkMsg) {
        let (_, mut w) = self.socket.split();

        println!("process_msg:got {:?}", msg);
        match msg.event {
            MsgEvent::TxsOfAddr { addr } => {
                let ser_txs = {
                    let shared_block_chain = self.shared_block_chain.lock().unwrap();
                    let addr_txs = shared_block_chain.txs_of_addr(addr);
                    bincode::serialize(&addr_txs[..]).unwrap()
                };
                w.write_all(&ser_txs[..]).await;
                w.flush().await;
                println!("process_msg:ser_txs:sent");
            }
            _ => {}
        }
    }
}

async fn network(shared_block_chain: Arc<Mutex<BlockChain>>) {
    let (tx_term, mut rx_term) = oneshot::channel::<u8>();
    let listener = TcpListener::bind(LOCAL).await.unwrap();
    let (msg_tx, _) = broadcast::channel::<NetworkMsg>(16);
    let client_id = Arc::new(AtomicU8::new(0));

    let server_loop = async {
        println!("server listening on localhost:8080");
        loop {
            tokio::select! {
                conn_result = listener.accept() => {
                    if let Err(ref e) = conn_result {}
                    let (socket, addr) = conn_result.unwrap();
                    let shared_block_chain = shared_block_chain.clone();
                    let client_id = client_id.clone();
                    let loaded_client_id = client_id.load(Ordering::Relaxed);
                    let clone_msg_tx = msg_tx.clone();
                    let msg_recv = clone_msg_tx.subscribe();

                    tokio::spawn(async move {
                        let mut handler = SocketHandler::new(
                            loaded_client_id,
                            socket,
                            clone_msg_tx,
                            msg_recv,
                            shared_block_chain);
                        handler.process().await;
                    });

                    client_id.fetch_add(1, Ordering::Relaxed);
                },
                _ = &mut rx_term => { break; }
            }
        }
    };

    tokio::select! {
        _ = server_loop => {}
        _ = tokio::signal::ctrl_c() => {
            println!("stoping server.");
            tx_term.send(0);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::transaction::{Transaction, TxBuilder};
    use tokio::{
        io::AsyncWriteExt,
        time::{self, Duration},
    };

    #[tokio::test]
    async fn txs_of_addr() {
        // Steps #
        // Chain -> cons tx -> add to chain's mempool, -> minning -> add block: DONE
        // Network -> send msg::tx_of_addr -> Chain : iterate blocks to find txs of given addr
        let a = "A".to_string();
        let b = "B".to_string();
        let chain = Arc::new(Mutex::new(BlockChain::new()));

        let tx1 = TxBuilder::new(a.clone(), b.clone(), 1.0)
            .inputs(vec![])
            .outputs(vec![])
            .build()
            .expect("unable to create tx-1");
        let tx2 = TxBuilder::new(b.clone(), a.clone(), 0.5)
            .inputs(vec![])
            .outputs(vec![])
            .build()
            .expect("unable to create tx-2");

        chain.lock().unwrap().add_transaction(tx1);
        chain.lock().unwrap().add_transaction(tx2);
        chain.lock().unwrap().minning();

        tokio::spawn(async move {
            time::sleep(Duration::from_millis(500)).await;
            let mut socket = TcpStream::connect(LOCAL).await.unwrap();
            let (mut r, mut w) = socket.split();

            let read_fu = async move {
                println!("read_fu:parking");
                let mut buf = [0u8; 1024];
                let read_result = r.read(&mut buf).await;
                if let Ok(_) = read_result {
                    let addr_txs: Vec<Transaction> = bincode::deserialize(&buf).unwrap();
                    println!("read_fu:addr_txs {:?}", addr_txs);
                };
            };

            let send_fu = async move {
                // Network -> send msg::tx_of_addr -> Chain : iterate blocks to find txs of given addr
                let msg = NetworkMsg {
                    event: MsgEvent::TxsOfAddr { addr: a.clone() },
                    propagation: MsgPropagation::ToChain,
                };
                let ser_msg_result = bincode::serialize(&msg);
                assert!(ser_msg_result.is_ok());

                time::sleep(Duration::from_millis(500)).await;

                println!("send_fu:send");
                let ser_msg = ser_msg_result.unwrap();
                w.write_all(&ser_msg[..]).await;
                w.flush().await;

                // after flush msg: give some time for reader
                time::sleep(Duration::from_millis(500)).await;
            };

            let a = tokio::select! {
               _ = read_fu => {},
               _ = send_fu => {}
            };
        });

        network(chain).await

        // assert!(false)
    }

    #[tokio::test] // MsgEvent::PushTrx
    async fn broadcast_trx() {
        let chain = Arc::new(Mutex::new(BlockChain::new()));
        let network_handle = tokio::spawn(async { network(chain).await });

        // - spawn some connected tcpstream(s)
        // - broadcast msg to all connected
        for i in 0..5 {
            let _i = Arc::new(i);
            let _i = _i.clone();
            tokio::spawn(async move {
                eprintln!("socket:{} is connect", *_i);
                let socket = TcpStream::connect(LOCAL).await.unwrap();
                time::sleep(Duration::from_secs(5)).await; // keep alive for 5 secs

                eprintln!("socket:{} is droped", *_i);
            });
        }

        // - connect
        // - create bytes of TcpMessage
        // - send the bytes to socket
        tokio::spawn(async {
            let mut socket = TcpStream::connect(LOCAL).await.unwrap();
            let (_, mut wr) = socket.split();
            eprintln!("producer:connect");

            let msg = NetworkMsg {
                propagation: MsgPropagation::Broadcast,
                event: MsgEvent::PushTrx {
                    addr: "sender".to_string(),
                    tx_bytes: "trx_bytes".to_string().as_bytes().to_vec(),
                },
            };
            let msg_bytes = bincode::serialize(&msg).unwrap();

            time::sleep(Duration::from_secs(2)).await;

            println!("sending message:len {}", msg_bytes.as_slice().len());
            if let Err(e) = wr.write_all(msg_bytes.as_slice()).await {
                eprintln!("write:err {:?}", e);
            }
            if let Err(e) = wr.flush().await {
                eprintln!("flush:err {:?}", e);
            }

            println!("send message:done");
        });

        network_handle.await;
        assert!(false);
    }
}
