
use std::io::Read;
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

    let state = Arc::new(Mutex::new(state));
    let new_txs = Arc::new(Mutex::new(Vec::<Tx>::new()));

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
                        let mut new_txs = new_txs.lock().unwrap();
                        new_txs.push(tx);
                    },
                    Mined(block) => {
                        let mut state = state.lock().unwrap();
                        if let Ok(utxo_set) = state.verify_block(&state.utxo_set,
                            state.blocks.last().unwrap(), &block) {
                            state.utxo_set = utxo_set;
                            println!("New block accepted");
                        } else {
                            println!("New block rejected");
                        }
                    },
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

/*impl Encoder<ServerFrame> for CoinCodec {
    type Item = ClientFrame;
    type Error = io::Error;

    fn encode(&mut self, src: &mut BytesMut) ->
        Result<Option<Self::Item>, Self::Error> {

        if src.is_empty() {
            return Ok(None)
        };

        match bincode::deserialize(src.as_ref()) {
            Ok(frame) => frame,
            Err(_) => Self::Error()
        }
    }
}
*/
