
use k256::ecdsa::Signature;
use k256::{SecretKey, PublicKey};
use bincode::deserialize;
use futures::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::hash::Hash;
//use tokio_serde::{Serializer, Deserializer, Framed};
use std::sync::{Arc, Mutex};
use tokio_util::codec::{Framed};
use coin::block::*;
use coin::user::*;
use coin::frametype::*;

use std::fs;
use serde::*;
use tokio::{
    net::{TcpListener, TcpStream}
};


//I have no fucking idea what I'm doing when
//it comes to networking let's hope I can do this
#[tokio::main]
async fn main() {
    println!("Spender go brrrrrrrrrr");
    //
    // Connect to the server
    let stream = TcpStream::connect(format!("{SERVER_IP}:{PORT}")).await.unwrap();
    let mut framed = Framed::new(stream, MinerCodec);

    // Get version
    framed.send(ClientFrame::GetVersion).await.unwrap();
    if let Some(Ok(ServerFrame::Version(version))) = framed.next().await {
        println!("Server version: {}", version);
    }

    let serialized = fs::read("state.bin").expect("Error reading file");
    let mut state: State = bincode::deserialize(&serialized).expect("Error deserializing");
    state.utxo_set = state.verify_all_blocks().unwrap();
    state.old_utxo_set = state.utxo_set.clone();

    //use my own key here
    let (signing, verifying) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

    let mut new_block = Block::new();
    let user = User::random();
    for _ in 0..10 {
        new_block.transact(&mut state.utxo_set, &signing, &user.verifying, 5).unwrap();
    }
    new_block.prev_hash = state.blocks.last().unwrap().get_hash();
    new_block.nonce = new_block.mine();
    assert!(state.verify_block(&state.old_utxo_set, &state.blocks.last().unwrap(), &new_block).is_ok());
    println!("Block successfully verified!");
    //state.blocks.push(new_block.clone());
    //assert!(state.verify_all_blocks().is_ok());
    //assert!(state.verify_block(&state.utxo_set, &state.blocks.last().unwrap(), &new_block).is_ok());
    framed.send(ClientFrame::TxFrame(new_block.txs)).await;
    println!("Spender submitting 10 transactions");

}
