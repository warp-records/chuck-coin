

use futures::{SinkExt, StreamExt};
//use tokio_serde::{Serializer, Deserializer, Framed};
//use tokio_util::codec::{Framed};
use coin::block::*;
use coin::user::*;
use coin::frametype::*;
use std::collections::HashMap;
use std::fs;
use tokio_tungstenite::{connect_async, tungstenite, tungstenite::protocol::Message};
use tungstenite::{ http::{Method, Request}, client::*};
use url::Url;
use bincode::{serialize, deserialize};
use bytes::Bytes;

//I have no fucking idea what I'm doing when
//it comes to networking let's hope I can do this

//creates 10 groups of 10 transactions sent from
//me to random wallet addresses
#[tokio::main]
async fn main() {
    println!("Spender go brrrrrrrrrr");
    //
    // Connect to the server
    let url = format!("ws://{SERVER_IP}:{PORT}");
    let ws_stream = connect_async(url.as_str().into_client_request().unwrap()).await.unwrap().0;
    let (mut write, mut read) = ws_stream.split();

    // Get version
    let get_version_msg = serialize(&ClientFrame::GetVersion).unwrap();
    write.send(Message::Binary(Bytes::from(get_version_msg))).await.unwrap();
    if let Some(Ok(Message::Binary(response))) = read.next().await {
        if let Ok(ServerFrame::Version(version)) = deserialize(&response) {
            println!("Server version: {}", version);
        }
    }

    //let serialized = fs::read("state.bin").expect("Error reading file");
    //let mut state: State = bincode::deserialize(&serialized).expect("Error deserializing");
    let get_blockchain_msg = serialize(&ClientFrame::GetBlockchain).unwrap();
    write.send(Message::Binary(Bytes::from(get_blockchain_msg))).await.unwrap();
    let mut blockchain = Vec::new();
    while let Some(Ok(Message::Binary(response))) = read.next().await {
        if let Ok(ServerFrame::BlockChain(data)) = deserialize(&response) {
            blockchain = data;
            break;
        }
        //panic!("Expected blockchain frame");
    };
    let mut state = State {
        blocks: blockchain,
        utxo_set: HashMap::new(),
        old_utxo_set: HashMap::new(),
    };
    if state.verify_all_and_update().is_err() { panic!("ur fucked lmao"); }

    //use my own key here
    //for _ in 0..10 {
    let (signing, verifying) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

    let mut new_block = Block::new();
    //let user = User::from_priv("EEADCC3CEC9EC11F6B172C800F846AAD5AEE59D2308BE01429B82393ACDE46C8");
    let user = User::random();

    //server freezes when sending a lot of txs
    const NUM_TX: u64 = 20;
    for _ in 0..NUM_TX {
        new_block.transact(&mut state.utxo_set, &signing, &user.verifying, 5).unwrap();
    }
    new_block.prev_hash = state.blocks.last().unwrap().get_hash();
    new_block.nonce = new_block.mine();
    assert!(state.add_block_if_valid(new_block.clone()).is_ok());
    println!("Block successfully verified!");

    println!("Submitting {NUM_TX} test transactions");
    let tx_frame_msg = serialize(&ClientFrame::TxFrame(new_block.txs.clone())).unwrap();
    write.send(Message::Binary(Bytes::from(tx_frame_msg))).await.unwrap();
    println!("Sent");
}
