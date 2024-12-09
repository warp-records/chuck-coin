
use crate::block::*;
use crate::tx::*;
use crate::user::*;
use serde::*;
use std::io;
use tokio_util::codec::{Decoder, Encoder, Framed};
use bytes::{BytesMut, Buf, BufMut};
use futures::{SinkExt, StreamExt};

//sent from client
#[derive(Serialize, Deserialize)]
pub enum ClientFrame {
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
pub enum ServerFrame {
    //idk if we'll need these two
    NewBlockMined,
    //Read this from cargotoml
    Version(String),
    //Client gets to decide which txs to mine
    NewTxPool(Vec<Tx>),
}

//consider merging or using a macro
pub struct MinerCodec;

impl Decoder for MinerCodec {
    type Item = ServerFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() { return Ok(None) }

        bincode::deserialize(&src[..])
            .map(|frame| { src.clear(); Some(frame) })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

impl Encoder<ClientFrame> for MinerCodec {
    type Error = io::Error;

    fn encode(&mut self, item: ClientFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = bincode::serialize(&item)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        dst.extend_from_slice(&bytes);
        Ok(())
    }
}

pub struct ServerCodec;

impl Decoder for ServerCodec {
    type Item = ClientFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() { return Ok(None) }

        bincode::deserialize(&src[..])
            .map(|frame| { src.clear(); Some(frame) })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}


impl Encoder<ServerFrame> for ServerCodec {
    type Error = io::Error;

    fn encode(&mut self, item: ServerFrame, dst: &mut BytesMut) ->
        Result<(), Self::Error> {

            let bytes = bincode::serialize(&item)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            dst.extend_from_slice(&bytes);
            Ok(())
        }
}
