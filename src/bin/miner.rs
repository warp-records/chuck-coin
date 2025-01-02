
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
use std::collections::HashMap;
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

    framed.send(ClientFrame::GetBlockchain).await;
    let Some(Ok(ServerFrame::BlockChain(blockchain))) = framed.next().await else {
        panic!("rip");
    };
    let mut state = State {
        blocks: blockchain,
        utxo_set: HashMap::new(),
        old_utxo_set: HashMap::new(),
    };
    if state.verify_all_and_update().is_err() { panic!("ur fucked lmao"); }

    loop {
        let mut server_txs = Vec::new();

        framed.send(ClientFrame::GetNewTxpool).await;
        while let Some(Ok(ServerFrame::NewTxPool(txs))) = framed.next().await {
            if !txs.is_empty() {
                server_txs = txs;
                break;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                framed.send(ClientFrame::GetNewTxpool).await.unwrap();
            }

            println!("Requesting new txpool");
        }

        //while let Some(Ok(ServerFrame::NewTxPool(_))) = framed.next().await {}
        //get hash to use for mining

        let mut new_block = Block::new();
        new_block.txs = server_txs;
        new_block.prev_hash = state.blocks.last().unwrap().get_hash();
        new_block.nonce = new_block.mine();
        assert!(state.add_block_if_valid(new_block.clone()).is_ok());

        println!("sending");
        framed.send(ClientFrame::Mined(new_block)).await.unwrap();
    }
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
