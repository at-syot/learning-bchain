use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, oneshot};

const LOCAL: &str = "127.0.0.1:8080";

#[derive(Clone, Serialize, Deserialize, Debug)]
enum TcpMessageTypes {
    Producer,
    Consumer,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct TcpMessage {
    message_type: TcpMessageTypes,
    content: String,
}

type MessageTx = broadcast::Sender<TcpMessage>;
type MessageRecv = broadcast::Receiver<TcpMessage>;
struct SocketHandler {
    client_id: u8,
    socket: TcpStream,
    producer: MessageTx,
    consumer: MessageRecv,
}

impl SocketHandler {
    pub fn new(
        client_id: u8,
        socket: TcpStream,
        producer: MessageTx,
        consumer: MessageRecv,
    ) -> Self {
        SocketHandler {
            client_id,
            socket,
            producer,
            consumer,
        }
    }

    pub async fn process(&mut self) {
        eprintln!("process:cid {}", self.client_id);
        let (mut rd, _) = self.socket.split();
        let mut buf = [0u8; 64];

        loop {
            tokio::select! {
                rd_result = rd.read(&mut buf) => {
                    match rd_result {
                        Err(_) => {
                            eprintln!("process:bytes {}, cid {}", buf.len(), self.client_id);
                            break;
                        }
                        Ok(0) => {
                            println!("process:socker closed || dropped");
                            break;
                        }
                        Ok(_) => {
                            let tcp_msg_result: bincode::Result<TcpMessage> =
                                bincode::deserialize(&buf[..]);
                            if let Ok(t) = tcp_msg_result {
                                println!("process:{}:tcpmsg:send {:?}", self.client_id, t);
                                self.producer.send(t);
                            }
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
}

async fn p2p_network() {
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::sync::Arc;

    let (tx_term, mut rx_term) = oneshot::channel::<u8>();
    let listener = TcpListener::bind(LOCAL).await.unwrap();
    let (msg_tx, _) = broadcast::channel::<TcpMessage>(16);
    let client_id = Arc::new(AtomicU8::new(0));

    let server_loop = async {
        println!("server listening on localhost:8080");
        loop {
            tokio::select! {
                conn_result = listener.accept() => {
                    if let Err(ref e) = conn_result {}
                    let (socket, addr) = conn_result.unwrap();
                    let client_id = client_id.clone();
                    let loaded_client_id = client_id.load(Ordering::Relaxed);
                    let clone_msg_tx = msg_tx.clone();
                    let msg_recv = clone_msg_tx.subscribe();

                    tokio::spawn(async move {
                        let mut handler = SocketHandler::new(loaded_client_id, socket, clone_msg_tx, msg_recv);
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
            dbg!("stoping server.");
            tx_term.send(0);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::{
        io::AsyncWriteExt,
        time::{self, Duration},
    };

    #[tokio::test]
    async fn p2p() {
        let p2p_handle = tokio::spawn(async { p2p_network().await });

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

            let msg = TcpMessage {
                message_type: TcpMessageTypes::Producer,
                content: "to all!!!".to_string(),
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

        // handle.await;
        p2p_handle.await;
        assert!(false);
    }
}
