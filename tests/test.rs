

use k256::{PublicKey, ecdsa::{SigningKey, VerifyingKey}};

pub fn keys_from_str(priv_key: &str) -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::from_bytes(hex::decode(priv_key).unwrap().as_slice().into()).unwrap();
    let verifying_key = VerifyingKey::from(signing_key.clone());

    (signing_key, verifying_key)
}

//#![allow(unused_imports)]
use coin::block::*;
use coin::tx::*;
use coin::user::*;
use k256::ecdsa::Signature;
use std::time::{SystemTime, UNIX_EPOCH};

use coin::block::*;
use coin::tx::*;
use coin::user::*;
use std::fs;

// test cases written by Deepseek R1
#[cfg(test)]
mod tests {
    use super::*;

    // Helper to load genesis keypair from file
    fn genesis_keys() -> (SigningKey, VerifyingKey) {
        let priv_key = fs::read_to_string("private_key.txt")
            .expect("private_key.txt needed for testing");
        keys_from_str(&priv_key.trim())
    }

    #[test]
    fn test_genesis_block_initialization() {
        let genesis = Block::genesis_block();
        let state = State::with_genesis_block();

        let (_, genesis_pub) = genesis_keys();
        let genesis_pub = PublicKey::from(genesis_pub);

        // Verify genesis output belongs to our key
        let genesis_out = &state.utxo_set.values().next().unwrap();
        assert_eq!(genesis_out.recipient, genesis_pub);
    }

    #[test]
    fn test_valid_transaction_flow() {
        let mut state = State::with_genesis_block();
        let (sender_priv, sender_pub) = genesis_keys();
        let recipient = User::random();

        let mut new_block = Block::new();
        new_block.prev_hash = state.blocks[0].get_hash();

        // Spend from genesis output
        new_block
            .transact(
                &mut state.utxo_set,
                &sender_priv,
                &recipient.verifying,
                50,
            )
            .expect("Failed to create transaction");

        new_block.nonce = new_block.mine();
        state.add_block_if_valid(new_block.clone())
            .expect("Block should be valid");

        // Verify balances
        let genesis_balance = state.get_balance(sender_pub.into());
        let recipient_balance = state.get_balance(recipient.verifying.into());

        assert_eq!(genesis_balance, Block::START_SUPPLY - 50);
        assert_eq!(recipient_balance, 50);
    }

    #[test]
    fn test_invalid_transaction_overspend() {
        let mut state = State::with_genesis_block();
        let (sender_priv, _) = genesis_keys();
        let recipient = User::random();

        let mut new_block = Block::new();
        let result = new_block.transact(
            &mut state.utxo_set,
            &sender_priv,
            &recipient.verifying,
            Block::START_SUPPLY + 1,
        );

        assert!(result.is_err(), "Should reject overspending");
    }

    #[test]
    fn test_utxo_set_update() {
        let mut state = State::with_genesis_block();
        let (sender_priv, _) = genesis_keys();
        let genesis_outpoint = Outpoint(state.blocks[0].txs[0].txid, 0);

        let mut new_block = Block::new();
        new_block.prev_hash = state.blocks[0].get_hash();

        new_block
            .transact(&mut state.utxo_set, &sender_priv, &sender_priv.verifying_key(), 10)
            .unwrap();

        new_block.nonce = new_block.mine();
        state.add_block_if_valid(new_block).unwrap();

        assert!(!state.utxo_set.contains_key(&genesis_outpoint));
    }

    // Other tests remain the same as they don't depend on keys
    #[test]
    fn test_block_mining_and_verification() {
        let mut block = Block::new();
        block.prev_hash = BLANK_BLOCK_HASH;
        block.time_stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Mine and verify work
        block.nonce = block.mine();
        assert!(
            block.verify_work(),
            "Mined block should pass work verification"
        );
    }

    #[test]
    fn test_block_rejection_bad_nonce() {
        let mut state = State::with_genesis_block();
        let mut invalid_block = Block::new();
        invalid_block.prev_hash = state.blocks[0].get_hash();
        invalid_block.nonce = 12345; // Clearly invalid nonce

        assert!(
            state.add_block_if_valid(invalid_block).is_err(),
            "Block with invalid nonce should be rejected"
        );
    }
}
