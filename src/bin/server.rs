
use futures::SinkExt;
use std::hash::Hash;
use std::io::Read;
use std::collections::HashSet;
//use tokio_serde::{Serializer, Deserializer, Framed};
use std::sync::{Arc, Mutex};
use serde::*;
use tokio_util::codec::{Decoder, Encoder, Framed};
use futures::StreamExt;
use bytes::{BytesMut, Buf, BufMut};
use std::io;
use coin::block::*;
use coin::tx::*;
use ClientFrame::*;
use ServerFrame::*;

use std::fs;
use serde::*;
use tokio::{
    net::{TcpListener, TcpStream}
};

//enum ConnectionType {
//    Spender,
//    Miner,
//}

//sent from client
#[derive(Serialize, Deserialize)]
enum ClientFrame {
    //ConnectionType,
    TxFrame(Tx),
    Mined(Block),
    GetBlockchain,
    GetLastBlock,
    GetNewTxpool,
    GetVersion,
}

#[derive(Serialize, Deserialize)]
//sent from server
enum ServerFrame {
    //idk if we'll need these two
    NewBlockMined,
    //Read this from cargotoml
    Version(String),
    //Client gets to decide which txs to mine
    NewTxPool(Vec<Tx>),
}

struct CoinCodec;

#[tokio::main]
async fn main() {
    println!("Starting server");
    let serialized = fs::read("state.bin").expect("Error reading file");
    let state: State = bincode::deserialize(&serialized).expect("Error deserializing");
    assert!(state.verify_all_blocks().is_ok());

    //wish there was an arcmutex macro or something

    let state = Arc::new(Mutex::new(state));
    //need hashmap since we're
    let new_txs = Arc::new(Mutex::new(HashSet::<Tx>::new()));

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection from: {}", addr);

        let mut framed_stream = Framed::new(socket, CoinCodec);
        let new_txs = new_txs.clone();
        let state = state.clone();
        tokio::spawn(async move {

            while let Some(Ok(frame)) = framed_stream.next().await {
                match frame {
                    TxFrame(tx) => {
                        println!("New tx received");
                        let mut new_txs = new_txs.lock().unwrap();
                        new_txs.insert(tx);
                    },
                    Mined(block) => {
                        let mut state = state.lock().unwrap();
                        let block_clone = block.clone();
                        if state.add_block_if_valid(block).is_ok() {
                                println!("New block accepted");
                                let mut new_txs = new_txs.lock().unwrap();
                                new_txs.retain(|item| !block_clone.txs.iter().any(|x| x == item));
                        } else {
                            println!("New block rejected");
                        }
                    },
                    GetNewTxpool => {
                        println!("Tx pool requested");
                        let new_txs = {
                            let new_txs = new_txs.lock().unwrap();
                            new_txs.iter().cloned().collect::<Vec<_>>()
                        };
                        framed_stream.send(ServerFrame::NewTxPool(new_txs)).await;
                    },
                    GetVersion => {
                        framed_stream.send(ServerFrame::Version(env!("CARGO_PKG_VERSION").to_string())).await;
                    },

                    //do later
                    _ => {
                        unimplemented!();
                    }
                }
            }

        });
        //match

    }
}


impl Decoder for CoinCodec {
    type Item = ClientFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) ->
        Result<Option<Self::Item>, Self::Error> {

        if src.is_empty() {
            return Ok(None)
        };

        match bincode::deserialize(&src[..]) {
            Ok(frame) => Ok(Some(frame)),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }
}

impl Encoder<ServerFrame> for CoinCodec {
    type Error = io::Error;

    fn encode(&mut self, item: ServerFrame, dst: &mut BytesMut) ->
        Result<(), Self::Error> {

            let bytes = bincode::serialize(&item)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            dst.extend_from_slice(&bytes);
            Ok(())
        }
}
