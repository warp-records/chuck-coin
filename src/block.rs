use crate::tx::*;
use ethnum::*;
use rust_decimal::*;
use rust_decimal_macros::*;

pub struct Block {
    prev_block: Option<Box<Block>>,
    prev_hash: u256,
    nonce: u256,
    inputs: Vec<TxOutput>,
    outputs: Vec<TxOutput>,
}

impl Block {
    const TOTAL_SUPPLY: Decimal = dec!(69);
    pub fn verify(&self) -> bool {
        //verify hashes
        //keep track o f
        true
    }
}
