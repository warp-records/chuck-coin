
use std::env::consts::OS;
//use tokio_serde::{Serializer, Deserializer, Framed};
use std::sync::{Arc, Mutex};
use tokio_util::codec::{Framed};
use futures::{StreamExt, SinkExt};
use coin::block::*;
use coin::tx::*;
use coin::frametype::*;
use ClientFrame::*;
use tokio_tungstenite::*;
use tungstenite::*;
use bytes::Bytes;

use std::fs;
use tokio::{join, net::TcpListener};

//enum ConnectionType {
//    Spender,
//    Miner,
//}

#[tokio::main]
async fn main() {
    println!("Starting server");
    let serialized = fs::read("state.bin").expect("Error reading file");
    let mut state: State = bincode::deserialize(&serialized).expect("Error deserializing");
    state.utxo_set = state.verify_all_blocks().unwrap();
    state.old_utxo_set = state.utxo_set.clone();

    //wish there was an arcmutex macro or something

    let state = Arc::new(Mutex::new(state));
    //need hashmap since we're
    //might have to track these as "tx groups" instead
    //due to dependencies

    //TODO: use txgroups to prevent repeat txs
    let new_txs = Arc::new(Mutex::new(Vec::<Vec::<Tx>>::new()));

    let listener = TcpListener::bind(format!("0.0.0.0:{PORT}")).await.unwrap();

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("New connection from: {addr}");

        let new_txs = new_txs.clone();
        let state = state.clone();

        let new_task = tokio::spawn(async move {
            let mut ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            //let (read, write) = ws_stream.split();


            while let Some(Ok(protocol::Message::Binary(msg))) = ws_stream.next().await {
                let mut frame = bincode::deserialize(&msg).unwrap();
                match frame {
                    TxFrame(txs) => {
                        println!("New txs received");
                        //todo: verify that txs are valid
                        let mut new_txs = { new_txs.lock().unwrap() };
                        new_txs.push(txs);
                    },
                    Mined(block) => {
                        let mut state = state.lock().unwrap();
                        let num_txs = block.txs.len();
                        if state.add_block_if_valid(block).is_ok() {
                                println!("New block accepted");
                                let mut new_txs = new_txs.lock().unwrap();
                                new_txs.clear();
                                assert!(state.verify_all_blocks().is_ok());
                                //let mut block_clone = block.clone();
                                //new_txs.retain(|item| !block_clone.txs.iter().any(|x| x == item));
                        } else {
                            println!("New block rejected");
                        }
                    },
                    GetNewTxpool => {
                        //println!("Tx pool requested");
                        let new_txs = { new_txs.lock().unwrap().clone() };
                        let serialized = bincode::serialize(&ServerFrame::NewTxPool(new_txs)).unwrap();
                        //ws_stream.send(Message::Binary(Bytes::from(serialized))).await;
                        ws_stream.send(Message::Binary(Bytes::from(serialized))).await.unwrap();
                    },
                    GetVersion => {
                        let serialized = bincode::serialize(&ServerFrame::Version(env!("CARGO_PKG_VERSION").to_string())).unwrap();
                        ws_stream.send(Message::Binary(Bytes::from(serialized))).await;
                    },
                    GetLastHash => {
                        println!("Last hash requested");
                        let last_hash = {
                            //change this later
                            let blocks = state.lock().unwrap().blocks.clone();
                            blocks.last().unwrap().get_hash()
                        };

                        //bincode::serialize(&)
                        let serialized = bincode::serialize(&ServerFrame::LastBlockHash(last_hash)).unwrap();
                        ws_stream.send(Message::Binary(Bytes::from(serialized))).await;
                    },
                    GetBlockchain => {
                        println!("Blockchain requested");
                        let block_chain = { state.lock().unwrap().blocks.clone() };

                        let serialized = bincode::serialize(&ServerFrame::BlockChain(block_chain)).unwrap();
                        ws_stream.send(Message::Binary(Bytes::from(serialized))).await;
                    }

                }
            }

        });

        //match

    }
}
