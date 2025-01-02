
use crate::block::*;
use crate::tx::*;
use serde::*;
use std::io;
use tokio_util::codec::{Decoder, Encoder};
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
pub struct MinerCodec;

impl Decoder for MinerCodec {
    type Item = ServerFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 { return Ok(None) }

        let msg_len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        if src.len() < 4 + msg_len { return Ok(None) }

        let msg = src[4..4+msg_len].to_vec();
        src.advance(4 + msg_len);

        bincode::deserialize(&msg)
            .map(Some)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

impl Encoder<ClientFrame> for MinerCodec {
    type Error = io::Error;

    fn encode(&mut self, item: ClientFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = bincode::serialize(&item)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let len = msg.len() as u32;
        dst.extend_from_slice(&len.to_be_bytes());
        dst.extend_from_slice(&msg);
        Ok(())
    }
}

pub struct ServerCodec;

impl Decoder for ServerCodec {
    type Item = ClientFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 { return Ok(None) }

        let msg_len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        if src.len() < 4 + msg_len { return Ok(None) }

        let msg = src[4..4+msg_len].to_vec();
        src.advance(4 + msg_len);

        bincode::deserialize(&msg)
            .map(Some)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

impl Encoder<ServerFrame> for ServerCodec {
    type Error = io::Error;

    fn encode(&mut self, item: ServerFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = bincode::serialize(&item)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let len = msg.len() as u32;
        dst.extend_from_slice(&len.to_be_bytes());
        dst.extend_from_slice(&msg);
        Ok(())
    }
}

/*
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
 */
