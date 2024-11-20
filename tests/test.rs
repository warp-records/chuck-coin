
use coin;
use coin::block::*;
use coin::tx::*;

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
    use std::{fs, iter::empty};

    use k256::PublicKey;

    use super::*;

    #[test]
    fn verify_blockchain() {
        let mut state = State {
            blocks: Vec::new(),
        };

        let mut block = Block {
            version: 0,
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };

        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        let MYYY_SPEECIAAALLL_KEEEYYY_FUCKYEAH = PublicKey::from(verifying_key);

        let root_txo = TxOutput {
            amount: Block::START_SUPPLY,
            spender: TxPredicate::Pubkey(MYYY_SPEECIAAALLL_KEEEYYY_FUCKYEAH),
            //I'M RICH
            recipient: MYYY_SPEECIAAALLL_KEEEYYY_FUCKYEAH,
        };

        let mut root_tx = Tx {
           inputs: Vec::new(),
           outputs: Vec::new(),
           txid: EMPTY_TXID,
           signature: signing_key.sign(&[]),
        };

        root_tx.outputs.push(root_txo);

        state.blocks.push(block);
        assert!(state.verify_all_blocks());
    }
}
