
use coin;
use coin::block::*;
use coin::tx::*;

use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    SecretKey,
};

#[cfg(test)]
mod tests {
    use std::iter::empty;

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

        let MYYY_SPEECIAAALLL_KEEEYYY_FUCKYEAH = PublicKey::from_sec1_bytes(hex::decode("04B0B5D59947A744C8ED5032F8B5EC77F56BFF09A7").unwrap().as_slice()).unwrap();
        let root_txo = TxOutput {
            amount: Block::START_SUPPLY,
            spender: TxPredicate::Pubkey(PublicKey::from_sec1_bytes(&[]).unwrap()),
            //I'M RICH
            recipient: MYYY_SPEECIAAALLL_KEEEYYY_FUCKYEAH,
        };

        let mut root_tx = Tx {
           inputs: Vec::new(),
           outputs: Vec::new(),
           txid: EMPTY_TXID,
           signature: Signature::from_slice(&[]).unwrap(),
        };

        root_tx.outputs.push(root_txo);

        state.blocks.push(block);
        assert!(state.verify_all_blocks());
    }
}
