
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
    use std::{collections::HashMap, env::consts::OS, fs, iter::empty, u64};

    use k256::PublicKey;

    use super::*;

    #[test]
    fn test_single_tx() {
        single_transaction();
    }

    #[test]
    fn test_multiple_txs() {
        multiple_transactions();
    }

    //have to pass the state of other tests to reuse the code
    //ToT
    fn first_block() -> State {
        let state = State::with_inital_block();
        assert!(state.verify_all_blocks().is_ok());

        state
    }

    fn single_transaction() -> (State, (SigningKey, VerifyingKey), (SigningKey, VerifyingKey)) {
        let mut state = first_block();

        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        let other_signing_key = SigningKey::random(&mut OsRng);
        let other_verifying_key = VerifyingKey::from(signing_key.clone());

        let mut new_block = Block {
            version: 0,
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };

        let tx_result = new_block.transact(
                &mut state.utxo_set,
                signing_key.clone(),
                PublicKey::from(other_verifying_key),
                1_000_000
        );
        assert!(tx_result.is_ok());

        new_block.nonce = new_block.mine();
        state.blocks.push(new_block);

        assert!(state.verify_all_blocks().is_ok());

        (state,
            (signing_key, verifying_key),
            (other_signing_key, other_verifying_key),
        )
    }

    fn multiple_transactions() {
        let (mut state,
            (signing_key, verifying_key),
            (other_signing_key, other_verifying_key),
        ) = single_transaction();

        let mut new_block = Block {
            version: 0,
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };

        let tx_result = new_block.transact(
                &mut state.utxo_set,
                signing_key.clone(),
                PublicKey::from(other_verifying_key),
                2_000_000
        );
        assert!(tx_result.is_ok());

        let tx_result = new_block.transact(
                &mut state.utxo_set,
                signing_key.clone(),
                PublicKey::from(other_verifying_key),
                u64::MAX,
        );
        assert!(tx_result.is_err());

        let tx_result = new_block.transact(
                &mut state.utxo_set,
                signing_key.clone(),
                PublicKey::from(other_verifying_key),
                20_000_000,
        );
        assert!(tx_result.is_ok());

        new_block.nonce = u64::MAX;
        state.blocks.push(new_block);

        assert!(state.verify_all_blocks().is_err());
    }
}
