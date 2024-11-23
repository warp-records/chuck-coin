#![allow(unused_imports)]
use bincode::serialize;
use hex_literal::hex;
use crate::block::*;
use crate::tx::*;
use serde::*;
use serde_json::*;
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

        let mut state = State::with_inital_block();
        let mut new_block = Block::new();

        //use my own key here
        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());
        let (other_signing, other_verifying) = create_keypair();
        let tx_result = new_block.transact(&mut state.utxo_set, signing_key, other_verifying.into(), 1_000_000);
        assert!(tx_result.is_ok());

        new_block.nonce = new_block.mine();
        assert!(state.verify_all_blocks().is_ok());

        state.blocks.push(new_block);

        let serialized = bincode::serialize(&state).expect("Error serializing");
        fs::write("state.bin", serialized).expect("Error writing to file");
        //fs::write("block.txt", serialized).expect("Error writing to file");

        let serialized = fs::read("state.bin").expect("Errir reading file");
        let state: State = bincode::deserialize(&serialized).expect("Error deserializing");
        assert!(state.verify_all_blocks().is_ok());
        println!("Serialiaze and deserialize successful!!! :D");
}

pub fn keys_from_str(priv_key: &str) -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::from_bytes(hex::decode(priv_key).unwrap().as_slice().into()).unwrap();
    let verifying_key = VerifyingKey::from(signing_key.clone());

    //println!("Private key: {} ", hex::encode_upper(signing_key.to_bytes()));
    //println!("Public key: {}", hex::encode_upper(verifying_key.to_sec1_bytes()));

    (signing_key, verifying_key)
}

pub fn create_keypair() -> (SigningKey, VerifyingKey) {
    use rand_core::OsRng;

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(signing_key.clone());
    //println!("Private key: {} ", hex::encode_upper(signing_key.to_bytes()));
    //println!("Public key: {}", hex::encode_upper(verifying_key.to_encoded_point(false).as_bytes()));
    (signing_key, verifying_key)
}
