// use core::block_chain::BlockChain;
use tokio::{io::{AsyncReadExt}, net::{TcpListener, TcpStream}};
use tokio::io::BufReader;
use std::{sync::{Arc, Mutex}};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
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

    let db: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("server starting on port: 8080");

    loop {
        let db = db.clone(); 
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            process(&mut socket).await;
        });
    }
}

async fn process(socket: &mut TcpStream) {
    let socket_addr = socket.peer_addr().unwrap();
    let client_ip = socket_addr.ip().to_string();

    let mut buff: [u8; 1024] = [0; 1024];
    let _ = socket.read(&mut buff).await;

    let trimmed_bytes: Vec<u8> = buff.iter().filter(|&&b| b != 0).cloned().collect();
    let inbound_raw = String::from_utf8(trimmed_bytes).unwrap();

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

    println!("command: M: {}, K: {}, Value: {}", method, key, value);

    // let msg = format!("Hello: {}", client_ip);
    // println!("writing to client {}", client_ip);
    // if let Err(e) = socket.write_all(msg.as_bytes()).await {
    //     eprintln!("fail to write to socket; err = {:?}", e);
    //     return;
    // }
}
