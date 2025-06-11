#![allow(unused_must_use, dead_code, unused_variables)]
mod core;
mod network;
mod p2p_network;

use core::block_chain::BlockChain;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::BufReader;
use tokio::sync::{mpsc, oneshot};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

fn old_main() {
    // let minner_address = Some(String::from("minner"));
    // let mut bc = BlockChain::new();
    // bc.block_chain_address = minner_address;
    // bc.inspect();
    //
    // bc.add_transaction("A".into(), "B".into(), 2.0);
    // bc.add_transaction("C".into(), "D".into(), 3.0);
    // bc.minning();
    // bc.inspect();
    //
    // bc.add_transaction("Me".into(), "Mom".into(), 1.0);
    // bc.minning();
    // bc.inspect();
}

// TODO:
// - create test network layer
//   features:
//      save connection to connnection pool (node pool),
//      sending message

type DB = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    network::test_accepting_conn().await;
}

async fn old_main_0() {
    let db: DB = Arc::new(Mutex::new(HashMap::new()));
    db.lock()
        .unwrap()
        .insert("foo".to_string(), "1".to_string());

    let server_addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(server_addr).await.unwrap();
    let listener_arc = Arc::new(listener);

    let (otx, mut orx) = oneshot::channel::<i32>();
    let listener_joinhandle = tokio::spawn(async move {
        loop {
            tokio::select! {
                received = &mut orx => {
                    if let Ok(i) = received {
                        println!(
                            "final db's value: {:?}",
                            db.clone().lock().unwrap().get("foo".into()).unwrap()
                        );
                    }
                    break;
                },
                result = listener_arc.accept() => {
                    match result {
                        Ok((mut socket, _)) => {
                            let db = db.clone();
                            tokio::spawn(async move {
                                process(&mut socket, db).await;
                            });
                        }
                        Err(e) => eprintln!("Error accepting connection: {}", e),
                    }
                }
            }
        }
    });

    let client_joinhandle = tokio::spawn(async move {
        for _ in 0..1000 {
            let mut socket = TcpStream::connect(server_addr).await.unwrap();
            let command = String::from("INC foo");
            if let Err(e) = socket.write_all(command.as_bytes()).await {
                eprintln!("Error sending command: {}", e);
            }
        }
    });

    match client_joinhandle.await {
        Err(e) => eprintln!("unable to join main thread: {:?}", e),
        Ok(_) => {
            println!("after joined");
            std::thread::sleep(std::time::Duration::from_secs(1));
            if let Err(e) = otx.send(0) {
                eprintln!("unable to stop the cleint {}", e);
            }
            println!("client gracfully stopped.")
        }
    }
}

async fn process(socket: &mut TcpStream, db: Arc<Mutex<HashMap<String, String>>>) {
    let socket_addr = socket.peer_addr().unwrap();
    let client_ip = socket_addr.ip().to_string();

    let mut buff: [u8; 1024] = [0; 1024];
    let _ = socket.read(&mut buff).await;

    let trimmed_bytes: Vec<u8> = buff.iter().filter(|&&b| b != 0).cloned().collect();
    let inbound_raw: String = String::from_utf8(trimmed_bytes).unwrap();

    // formatting
    let command_raw = inbound_raw.split("\r\n").last().unwrap().to_string();
    let command: Vec<&str> = command_raw.split(' ').collect();

    let method = *command.first().unwrap();
    let mut key: &str = command.last().unwrap();
    let mut value: &str = "";
    if command.len() > 2 {
        let mut com_iter = command.iter();
        let _ = com_iter.next();
        key = com_iter.next().unwrap();
        value = com_iter.next().unwrap();
    }

    // println!("command: M: {}, K: {}, Value: {}", method, key, value);
    let mut db = db.lock().unwrap();
    if method == "GET" {
        if let Some(v) = db.get(key) {
            println!("retrived value: {:?}", v);
            socket.write_all(v.clone().as_bytes());
        }
        socket.write_u8(0);
        return;
    }

    let old_v = db.get(key.into()).unwrap();
    let old_u8: i32 = old_v.parse().unwrap();
    let next_v = old_u8 + 1;
    db.insert(key.to_string(), next_v.to_string());
    // println!("save key: {}, value: {} - success!", key, next_v);
}
