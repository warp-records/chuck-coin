
use coin;
use coin::block::*;
use coin::tx::*;
use std::fs;
use rand_core::OsRng;
use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    SecretKey,
};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use serde::*;

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

    #[test]
    fn test_bad_nonce() {
        bad_nonce();
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
        //HOW THE FUCK did I mess this up
        let other_verifying_key = VerifyingKey::from(other_signing_key.clone());

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
                10_000_000
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

    fn multiple_transactions() -> (State, (SigningKey, VerifyingKey), (SigningKey, VerifyingKey)) {
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
                other_signing_key.clone(),
                PublicKey::from(verifying_key),
                1_000_000
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
                10_000_000,
        );
        assert!(tx_result.is_ok());

        let tx_result = new_block.transact(
                &mut state.utxo_set,
                other_signing_key.clone(),
                PublicKey::from(verifying_key),
                2_000_000,
        );
        print!("ok");
        assert!(tx_result.is_ok());

        //15 mil by now
        //anotheeerrr test
        let tx_result = new_block.transact(
                &mut state.utxo_set,
                other_signing_key.clone(),
                PublicKey::from(verifying_key),
                5_000_000,
        );

        let tx_result = new_block.transact(
                &mut state.utxo_set,
                other_signing_key.clone(),
                PublicKey::from(verifying_key),
                100_000_000,
        );
        assert!(tx_result.is_err());

        new_block.nonce = new_block.mine();
        state.blocks.push(new_block);
        let result = state.verify_all_blocks();

        assert!(result.is_ok());
        (state, (signing_key, verifying_key), (other_signing_key, other_verifying_key))

    }

    pub fn bad_nonce() -> (State, (SigningKey, VerifyingKey), (SigningKey, VerifyingKey)) {
        let (mut state, (signing_key, verifying_key), (other_signing_key, other_verifying_key)) = multiple_transactions();

        let mut last_block = state.blocks.pop().unwrap();
        last_block.nonce = 0;
        state.blocks.push(last_block);
        assert!(state.verify_all_blocks().is_err());

        last_block = state.blocks.pop().unwrap();
        last_block.nonce = last_block.mine();
        state.blocks.push(last_block);
        assert!(state.verify_all_blocks().is_ok());

        (state, (signing_key, verifying_key), (other_signing_key, other_verifying_key))
    }


    //thanks claude!!!
    #[test]
    fn test_tx_input_serde() {
        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        let signature = signing_key.sign(b"test message");
        let outpoint = Outpoint([1u8; 32], 0);

        let tx_input = TxInput {
            signature,
            prev_out: outpoint
        };

        let serialized = serde_json::to_string(&tx_input).unwrap();
        let deserialized: TxInput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(tx_input.prev_out.0, deserialized.prev_out.0);
        assert_eq!(tx_input.prev_out.1, deserialized.prev_out.1);
        assert_eq!(tx_input.signature.to_bytes(), deserialized.signature.to_bytes());
    }
}


#[test]
fn test_tx_output_serde() {
    let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

    let tx_output = TxOutput {
        spender: TxPredicate::Pubkey(verifying_key.into()),
        amount: 1000,
        recipient: verifying_key.into(), // Using same key for test simplicity
    };

    let serialized = serde_json::to_string(&tx_output).unwrap();
    let deserialized: TxOutput = serde_json::from_str(&serialized).unwrap();

    assert_eq!(tx_output.amount, deserialized.amount);
    assert_eq!(
        tx_output.spender.unwrap_key().to_encoded_point(false).as_bytes(),
        deserialized.spender.unwrap_key().to_encoded_point(false).as_bytes()
    );
    assert_eq!(
        tx_output.recipient.to_encoded_point(false).as_bytes(),
        deserialized.recipient.to_encoded_point(false).as_bytes()
    );
}

#[test]
fn test_tx_serde() {
    let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

    // Create a test transaction
    let mut tx = Tx::new();

    // Add an input
    let signature = signing_key.sign(b"test message");
    let outpoint = Outpoint([1u8; 32], 0);
    tx.inputs.push(TxInput {
        signature,
        prev_out: outpoint,
    });

    // Add an output
    tx.outputs.push(TxOutput {
        spender: TxPredicate::Pubkey(verifying_key.into()),
        amount: 1000,
        recipient: verifying_key.into(),
    });

    // Set txid and signature
    tx.txid = tx.get_txid();
    tx.signature = signing_key.sign(&tx.txid);

    // Test serialization/deserialization
    let serialized = serde_json::to_string(&tx).unwrap();
    let deserialized: Tx = serde_json::from_str(&serialized).unwrap();

    assert_eq!(tx.txid, deserialized.txid);
    assert_eq!(tx.signature.to_bytes(), deserialized.signature.to_bytes());
    assert_eq!(tx.inputs.len(), deserialized.inputs.len());
    assert_eq!(tx.outputs.len(), deserialized.outputs.len());
}


fn test_block_serde() {
    let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

    // Create a test block
    let mut block = Block::new();
    block.version = 1;
    block.prev_hash = 12345;
    block.nonce = 67890;

    // Add a transaction
    let mut tx = Tx::new();
    tx.inputs.push(TxInput {
        signature: signing_key.sign(b"test message"),
        prev_out: Outpoint([1u8; 32], 0),
    });
    tx.outputs.push(TxOutput {
        spender: TxPredicate::Pubkey(verifying_key.into()),
        amount: 1000,
        recipient: verifying_key.into(),
    });
    tx.txid = tx.get_txid();
    tx.signature = signing_key.sign(&tx.txid);

    block.txs.push(tx);

    // Test serialization/deserialization
    let serialized = serde_json::to_string(&block).unwrap();
    let deserialized: Block = serde_json::from_str(&serialized).unwrap();

    assert_eq!(block.version, deserialized.version);
    assert_eq!(block.prev_hash, deserialized.prev_hash);
    assert_eq!(block.nonce, deserialized.nonce);
    assert_eq!(block.txs.len(), deserialized.txs.len());
}
