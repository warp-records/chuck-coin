#![allow(unused_imports)]
use hex_literal::hex;
use crate::block::*;
use crate::tx::*;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant, SystemTime};
use sha3::*;
use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    SecretKey,
};
use rand_core::OsRng;

pub mod tx;
pub mod block;

fn main() {
        println!("Chuck coin: where a kid can be a kid!");
        println!("Take a coin kiddo:\n");
        println!("{}", fs::read_to_string("asciiart.txt").unwrap());

        let mut block = Block {
            version: 0,
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };

        //block.transact();

        //more fun to call it "gold" than nonce lol
        //get it, because you're mining it...
        let gold = block.mine();
        println!("Struck gold: 0x{:X}", gold);

}

pub fn keys_from_str(priv_key: &str) -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::from_bytes(hex::decode(priv_key).unwrap().as_slice().into()).unwrap();
    let verifying_key = VerifyingKey::from(signing_key.clone());

    (signing_key, verifying_key)
}

pub fn create_keypair() {
    use rand_core::OsRng;

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(signing_key.clone());
    println!("Private key: {} ", hex::encode_upper(signing_key.to_bytes()));
    println!("Public key: {}", hex::encode_upper(verifying_key.to_encoded_point(false).as_bytes()));
}

pub fn initial_block() -> Block {
        let signing_key = fs::read_to_string("priv_key.txt");
        let (signing_key, verifying_key) = keys_from_str(&signing_key.unwrap());

        let mut block = Block {
            version: 0,
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };


        /*
        let intial_txo = TxOutput {
            spender: 0x00,
            amount: Block::START_SUPPLY,
            recipient: verifying_key,
        };


        let initial_intput = TxInput {
            signature: signing_key.sign(prev_out),
            prev_out: Outpoint(EMPTY_TXID, 0)
        };
 */
        block
}
