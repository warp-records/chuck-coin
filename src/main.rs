#![allow(unused_imports)]

use crate::block::Block;
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

        let empty_block = Block {
            utxo_set: HashMap::new(),
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };

        let gold = empty_block.mine();
        println!("Struck gold: {:X}", gold);
}

pub fn create_keypair() {
    use rand_core::OsRng;

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(signing_key.clone());
    println!("Private key: {} ", hex::encode_upper(signing_key.to_bytes()));
    println!("Public key: {}", hex::encode_upper(verifying_key.to_encoded_point(false).as_bytes()));
}
