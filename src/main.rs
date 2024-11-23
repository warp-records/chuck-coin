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

struct User {
    signing: SigningKey,
    verifying: VerifyingKey,
}


fn main() {
        println!("Chuck coin: where a kid can be a kid!");
        println!("Take a coin kiddo:\n");
        println!("{}", fs::read_to_string("asciiart.txt").unwrap());

        let mut state = State::with_inital_block();
        let mut new_block = Block::new();

        //use my own key here
        let (signing, verifying) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());
        let me = User { signing, verifying };
        let user1 = User::random();
        let user2 = User::random();

        let tx_result = new_block.transact(&mut state.utxo_set, &me.signing, &user1.verifying, 2_000_000);
        assert!(tx_result.is_ok());
        let tx_result = new_block.transact(&mut state.utxo_set, &me.signing, &user1.verifying, 69_000_000);
        assert!(tx_result.is_err());

        let tx_result = new_block.transact(&mut state.utxo_set, &user1.signing, &user2.verifying, 1_000_000);
        assert!(tx_result.is_ok());

        let tx_result = new_block.transact(&mut state.utxo_set, &user2.signing, &me.verifying, 1_000_000);
        assert!(tx_result.is_ok());
        let tx_result = new_block.transact(&mut state.utxo_set, &user2.signing, &me.verifying, 1_000_000);
        assert!(tx_result.is_err());


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

impl User {
    pub fn random() -> Self {

        use rand_core::OsRng;

        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(signing_key.clone());
        Self { signing: signing_key, verifying: verifying_key }
    }
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
