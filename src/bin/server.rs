
use coin::block::*;
use coin::tx::*;

use std::fs;
use serde::*;
use tokio::{
    net::{TcpListener, TcpStream}
};

enum Message {
    TxMsg { msg: String },
    MinedMsg { hash: String },

}

#[tokio::main]
async fn main() {
    println!("Starting server");
    let serialized = fs::read("state.bin").expect("Error reading file");
    let state: State = bincode::deserialize(&serialized).expect("Error deserializing");
    let verify_result = state.verify_all_blocks();

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection from: {}", addr);

        //match

    }
}

async fn proc_miner() {

}

async fn proc_spender() {

}
