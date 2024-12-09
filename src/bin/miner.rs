
use k256::ecdsa::Signature;
use k256::{SecretKey, PublicKey};
use bincode::deserialize;
use futures::{SinkExt, StreamExt};
use std::collections::HashSet;
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

#[tokio::main]
async fn main() {
    // Connect to the server
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let mut framed = Framed::new(stream, MinerCodec);

    // Get version
    framed.send(ClientFrame::GetVersion).await.unwrap();
    if let Some(Ok(ServerFrame::Version(version))) = framed.next().await {
        println!("Server version: {}", version);
    }

    let serialized = fs::read("state.bin").expect("Error reading file");
    let mut state: State = bincode::deserialize(&serialized).expect("Error deserializing");
    state.utxo_set = state.verify_all_blocks().unwrap();

    let mut new_block = Block::new();

    //use my own key here
    let (signing, verifying) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

    //there was a test here before
    let user0 = User::random();

    new_block.transact(&mut state.utxo_set, &signing, &user0.verifying, 2).unwrap();
    new_block.prev_hash = state.blocks.last().unwrap().get_hash();
    new_block.nonce = new_block.mine();
    state.blocks.push(new_block);

    assert!(state.verify_all_blocks().is_ok());

    //framed.send(ClientFrame::TxFrame(new_block.txs[0].clone())).await.unwrap();
    framed.send(ClientFrame::Mined(state.blocks.last().unwrap().clone())).await;


    /*
    // Mining loop
    loop {
        // Get tx pool
        framed.send(ClientFrame::GetNewTxpool).await.unwrap();

        if let Some(Ok(ServerFrame::NewTxPool(txs))) = framed.next().await {
            if txs.is_empty() {
                println!("No transactions to mine");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }

            println!("Mining block with {} transactions", txs.len());

            // Create and mine a new block
            let mut block = Block::new();
            block.txs = txs;
            block.nonce = block.mine();

            println!("Found nonce: {}", block.nonce);

            // Submit mined block
            framed.send(ClientFrame::Mined(block)).await.unwrap();
        }
    } */
}
