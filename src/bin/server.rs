
//use tokio_serde::{Serializer, Deserializer, Framed};
use std::sync::{Arc, Mutex};
use tokio_util::codec::{Framed};
use futures::{StreamExt, SinkExt};
use coin::block::*;
use coin::tx::*;
use coin::frametype::*;
use ClientFrame::*;


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
    //let new_txs = Arc::new(Mutex::new(HashSet::<Tx>::new()));
    //TODO: use txgroups to prevent repeat txs
    let new_txs = Arc::new(Mutex::new(Vec::<Tx>::new()));

    let listener = TcpListener::bind(format!("0.0.0.0:{PORT}")).await.unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection from: {addr}");

        let mut framed_stream = Framed::new(socket, ServerCodec);
        let new_txs = new_txs.clone();
        let state = state.clone();
        let new_task = tokio::spawn(async move {

            while let Some(Ok(frame)) = framed_stream.next().await {
                match frame {
                    TxFrame(txs) => {
                        println!("New txs received");
                        //todo: verify that txs are valid
                        let mut new_txs = new_txs.lock().unwrap();
                        new_txs.extend(txs);
                    },
                    Mined(block) => {
                        let mut state = state.lock().unwrap();
                        let block_clone = block.clone();
                        if state.add_block_if_valid(block).is_ok() {
                                println!("New block accepted");
                                let mut new_txs = new_txs.lock().unwrap();
                                new_txs.clear();
                                //new_txs.retain(|item| !block_clone.txs.iter().any(|x| x == item));
                        } else {
                            println!("New block rejected");
                        }
                    },
                    GetNewTxpool => {
                        //println!("Tx pool requested");
                        let new_txs = { new_txs.lock().unwrap().clone() };
                        framed_stream.send(ServerFrame::NewTxPool(new_txs)).await;
                    },
                    GetVersion => {
                        framed_stream.send(ServerFrame::Version(env!("CARGO_PKG_VERSION").to_string())).await;
                    },
                    GetLastHash => {
                        println!("Last hash requested");
                        let last_hash = {
                            //change this later
                            let blocks = state.lock().unwrap().blocks.clone();
                            blocks.last().unwrap().get_hash()
                        };

                        framed_stream.send(ServerFrame::LastBlockHash(last_hash)).await;
                    },
                    GetBlockchain => {
                        println!("Blockchain requested");
                        let block_chain = { state.lock().unwrap().blocks.clone() };
                        framed_stream.send(ServerFrame::BlockChain(block_chain)).await;
                    }


                    //do later
                    _ => {
                        unimplemented!();
                    }
                }
            }

        });

        join!(new_task);
        //match

    }
}
