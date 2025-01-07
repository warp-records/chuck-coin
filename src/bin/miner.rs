#![forbid(unsafe_code)]
use bincode::deserialize;
use futures::{SinkExt, StreamExt};
use k256::ecdsa::Signature;
use k256::{PublicKey, SecretKey};
use std::collections::HashSet;
//use tokio_serde::{Serializer, Deserializer, Framed};
use coin::block::*;
use coin::frametype::*;
use coin::user::*;
use serde::*;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() {
    // Connect to the server
    let stream = TcpStream::connect(format!("{SERVER_IP}:{PORT}"))
        .await
        .unwrap();
    let mut framed = Framed::new(stream, MinerCodec);

    // Get version
    framed.send(ClientFrame::GetVersion).await.unwrap();
    if let Some(Ok(ServerFrame::Version(version))) = framed.next().await {
        println!("Server version: {}", version);
    }

    framed.send(ClientFrame::GetBlockchain).await;
    let mut blockchain = Vec::new();
    //just discard other frames for now, might have a
    //frame buffer in the future ( ˘ ³˘)
    while let Some(Ok(frame)) = framed.next().await {
        match frame {
            ServerFrame::BlockChain(data) => {
                blockchain = data;
                break;
            }
            _ => {
                continue;
            }
        }
        //panic!("Expected blockchain frame");
    }
    let mut state = State {
        blocks: blockchain,
        utxo_set: HashMap::new(),
        old_utxo_set: HashMap::new(),
    };
    if state.verify_all_and_update().is_err() {
        panic!("ur fucked lmao");
    }

    loop {
        let mut tx_groups = Vec::new();

        framed.send(ClientFrame::GetLastHash).await;
        let mut last_hash = BLANK_BLOCK_HASH;
        while let Some(Ok(frame)) = framed.next().await {
            match frame {
                ServerFrame::LastBlockHash(hash) => {
                    last_hash = hash;
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
        //kinda hacky wacky
        if last_hash != state.blocks.last().unwrap().get_hash() {
            state.blocks.pop();
        }

        framed.send(ClientFrame::GetNewTxpool).await;
        println!("Requesting new txpool");
        framed.send(ClientFrame::GetNewTxpool).await;
        while let Some(Ok(ServerFrame::NewTxPool(txs))) = framed.next().await {
            if !txs.is_empty() {
                tx_groups = txs;
                break;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                framed.send(ClientFrame::GetNewTxpool).await.unwrap();
            }
        }

        //while let Some(Ok(ServerFrame::NewTxPool(_))) = framed.next().await {}
        //get hash to use for mining

        let mut new_block = Block::new();
        new_block.prev_hash = state.blocks.last().unwrap().get_hash();
        let prev_block = &state.blocks.last().unwrap();

        let mut utxo_set = state.old_utxo_set.clone();
        for group in tx_groups {
            //idk if I should clone this lol
            new_block.txs.extend(group.clone());
            //new_block.nonce = new_block.mine();

            if let Ok(new_utxo_set) = state.verify_block(&utxo_set, &prev_block, &new_block, true) {
                utxo_set = new_utxo_set;
            } else {
                new_block.txs.truncate(new_block.txs.len() - group.len());
            }
        }
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
