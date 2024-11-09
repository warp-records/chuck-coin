use crate::tx::*;
use std::collections::HashMap;
use ethnum::*;
use serde::{Deserialize, Serialize};
use sha3::*;
use k256::{
    ecdsa::{SigningKey, signature::Verifier, VerifyingKey, Signature, signature::Signer},
    SecretKey,
    PublicKey,
};


//#[derive(Serialize, Deserialize, Debug)]
pub struct BlockList {
    blocks: Vec<Block>,
}

//#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    utxo_set: HashMap<Outpoint, TxOutput>,
    prev_hash: u256,
    nonce: u256,
    txs: Vec<Tx>,
}

//lol XD
//const TITTIES: Decimal = dec!(7177135);
//const BOOBIES: Decimal = dec!(8008135);
//consider using these constants in the project somehow

//this shit is hard
impl Block {
    const START_SUPPLY: u64 = 420 * 1_000_000;
    const TOTAL_SUPPLY: u64 = 69 * 1_000_000;

    pub fn verify_blockchain(&self) -> bool {
        //keep track of balances
        let mut input_total: u64 = 0;
        let mut output_total: u64 = 0;

        //verify hashes
        for tx in &self.txs {
            for (i, input) in tx.inputs.iter().enumerate() {
                let Some(prev_out) = self.utxo_set.get(&input.prev_out) else {
                    //uh oh...
                    return false;
                };

                if !Self::verify_sig(input.signature, &prev_out.spender, &tx, i as u64) {
                    //nice try hackers
                    return false;
                }

                input_total += input.amount;
            }

            for output in &tx.outputs {
                output_total += output.amount;
            }

            if input_total < output_total {
                return false;
            }
        }

        true
    }

    //check that signature equals the hash of tall transactions
    //and the transaction index combined, all signed by the spender
    fn verify_sig(sig: Signature, predicate: &TxPredicate, tx: &Tx, idx: u64) -> bool {
        //better hasher for cryptographic applications
        let mut hasher = Sha3_256::new();
        hasher.update(tx.as_bytes().as_slice());
        hasher.update(idx.to_be_bytes());
        let message = hasher.finalize();

        let verifying_key = VerifyingKey::from(predicate.unwrap_key());
        verifying_key.verify(&message[..], &sig).is_ok()
    }
}
