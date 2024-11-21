
use coin;
use coin::block::*;
use coin::tx::*;
use rand_core::OsRng;
use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    SecretKey,
};

pub fn keys_from_str(priv_key: &str) -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::from_bytes(hex::decode(priv_key).unwrap().as_slice().into()).unwrap();
    let verifying_key = VerifyingKey::from(signing_key.clone());

    (signing_key, verifying_key)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env::consts::OS, fs, iter::empty};

    use k256::PublicKey;

    use super::*;

    #[test]
    fn first_block() {
        let state = State::with_inital_block();
        assert!(state.verify_all_blocks());
    }

    #[test]
    fn test_transaction() {
        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        let second_signing_key = SigningKey::random(&mut OsRng);
        let second_verifying_key = VerifyingKey::from(signing_key.clone());

        let mut state = State::with_inital_block();

        let mut new_block = Block {
            version: 0,
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };

        let new_tx = new_block.transact(&mut state.utxo_set,
            signing_key.clone(),
            PublicKey::from(second_verifying_key),
            1_000_000,
        ).expect("TX Failed");

        //let new_tx = new_block.transact(&mut state.utxo_set,
       //     signing_key,
       //     PublicKey::from(second_verifying_key),
       //     2_000_000,
       // ).expect("TX Failed");

        state.blocks.push(new_block);

        assert!(state.verify_all_blocks());
    }
}
