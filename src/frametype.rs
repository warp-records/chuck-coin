
use crate::block::*;
use crate::tx::*;
use serde::*;
use std::io;
//use tokio_util::codec::{Decoder, Encoder};
use bytes::{Buf, BytesMut};
use futures::StreamExt;

//sent from client
#[derive(Serialize, Deserialize)]
pub enum ClientFrame {
    //ConnectionType,
    TxFrame(Vec<Tx>),
    Mined(Block),
    GetBlockchain,
    GetLastHash,
    GetNewTxpool,
    GetVersion,
}

#[derive(Serialize, Deserialize)]
//sent from server
pub enum ServerFrame {
    //idk if we'll need these two
    NewBlockMined,
    //Read this from cargotoml
    Version(String),
    //Client gets to decide which txs to mine
    NewTxPool(Vec<Vec<Tx>>),
    LastBlockHash(BlockHash),
    BlockChain(Vec<Block>),
}

//should probably move this to a config file
pub const PORT: u16 = 1337;
pub const SERVER_IP: &str = "129.213.163.237";
