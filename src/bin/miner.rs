
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

    let mut new_txs = Vec::new();
    framed.send(ClientFrame::GetNewTxpool).await;
    while let Some(Ok(ServerFrame::NewTxPool(txs))) = framed.next().await {
        if !txs.is_empty() {
            new_txs = txs;
            break;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            framed.send(ClientFrame::GetNewTxpool).await;
        }
    }

    framed.send(ClientFrame::GetLastHash).await;
    //shouldn't need this but lets see if it works with it first

    println!("Got prev block hash");
    let prev_hash = if let Some(Ok(ServerFrame::LastBlockHash(hash))) = framed.next().await {
        println!("Server hash: {:?}", hash);
        hash
    } else {
        panic!();
    };
    println!("Local hash: {:?}", state.blocks.last().unwrap().get_hash());

    let mut new_block = Block::new();
    new_block.txs = new_txs;
    new_block.prev_hash = prev_hash;
    new_block.nonce = new_block.mine();
    assert!(state.add_block_if_valid(new_block.clone()).is_ok());

    println!("sending");
    framed.send(ClientFrame::Mined(new_block)).await.unwrap();
}

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
