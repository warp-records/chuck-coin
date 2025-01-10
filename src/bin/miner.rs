

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::Bytes;
use url::Url;

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
    let url = format!("ws://{SERVER_IP}:{PORT}");
    let request = url.into_client_request().unwrap();
    let (ws_stream, _) = connect_async(request).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let frame = ClientFrame::GetVersion;
    let serialized = bincode::serialize(&frame).unwrap();
    ws_sender.send(Message::Binary(Bytes::from(serialized))).await.unwrap();
    if let Some(Ok(Message::Binary(data))) = ws_receiver.next().await {
        if let Ok(ServerFrame::Version(version)) = bincode::deserialize(&data) {
            println!("Server version: {}", version);
        }
    }

    let serialized = bincode::serialize(&ClientFrame::GetBlockchain).unwrap();
    ws_sender.send(Message::Binary(Bytes::from(serialized))).await.unwrap();
    let mut blockchain = Vec::new();
    //just discard other frames for now, might have a
    //frame buffer in the future ( ˘ ³˘)
    while let Some(Ok(Message::Binary(data))) = ws_receiver.next().await {
        if let Ok(ServerFrame::BlockChain(data)) = bincode::deserialize(&data) {
            blockchain = data;
            break;
        }
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

        let frame = ClientFrame::GetLastHash;
        let serialized = bincode::serialize(&frame).unwrap();
        ws_sender.send(Message::Binary(Bytes::from(serialized))).await.unwrap();
        let mut last_hash = BLANK_BLOCK_HASH;
        while let Some(Ok(Message::Binary(data))) = ws_receiver.next().await {
            if let Ok(ServerFrame::LastBlockHash(hash)) = bincode::deserialize(&data) {
                last_hash = hash;
                break;
            }
        }
        //kinda hacky wacky
        if last_hash != state.blocks.last().unwrap().get_hash() {
            state.blocks.pop();
        }

        let frame = ClientFrame::GetNewTxpool;
        let serialized_bytes = Bytes::from(bincode::serialize(&frame).unwrap());
        ws_sender.send(Message::Binary(serialized_bytes.clone())).await.unwrap();
        println!("Requesting new txpool");
        ws_sender.send(Message::Binary(Bytes::from(serialized_bytes.clone()))).await.unwrap();
        while let Some(Ok(Message::Binary(data))) = ws_receiver.next().await {
            if let Ok(ServerFrame::NewTxPool(txs)) = bincode::deserialize(&data) {
                if !txs.is_empty() {
                    tx_groups = txs;
                    break;
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    ws_sender.send(Message::Binary(serialized_bytes.clone())).await.unwrap();
                }
            }
        }

        //while let Some(Ok(ServerFrame::NewTxPool(_))) = ws_receiver.next().await {}
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

        println!("Sending {} transactions", new_block.txs.len());
        let frame = ClientFrame::Mined(new_block);
        let serialized = bincode::serialize(&frame).unwrap();
        ws_sender.send(Message::Binary(Bytes::from(serialized))).await.unwrap();
    }
}
