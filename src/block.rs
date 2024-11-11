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
//will prune blocks later
pub struct State {
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
//const TITTIES = 7177135;
//const BOOBIES = 8008135;
//consider using these as currency limits
//or something in the project

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
                //check that all inputs being used exited previously
                let Some(prev_out) = self.utxo_set.get(&input.prev_out) else {
                    //uh oh...
                    return false;
                };

                if !Self::verify_sig(input.signature, &prev_out.spender, &tx, i as u64) {
                    //nice try hackers
                    return false;
                }

                //pretty sure we DON'T have to check
                //the amount from each individual spender
                input_total += prev_out.amount;
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
        //make sure this is compatible with the way txids
       //are created in transact function
        tx.inputs.iter().for_each(|input| { hasher.update(input.as_bytes()); });
        tx.outputs.iter().for_each(|output| { hasher.update(output.as_bytes()); });
        hasher.update(idx.to_be_bytes());
        let message = hasher.finalize();

        let verifying_key = VerifyingKey::from(predicate.unwrap_key());
        verifying_key.verify(&message[..], &sig).is_ok()
    }

    //must be executed on the spenders hardware
    //since spender_priv is passed as an arugment
    pub fn transact(&mut self, spender_priv: SecretKey, recipient_pub: PublicKey, amount: u64) -> Result<Tx, ()> {
        let spender_pub = spender_priv.public_key();

        let mut balance: u64 = 0;
        //I hope I'm doing this right lol
        let mut spendable = Vec::new();
        for tx in &self.txs {
            for (i, old_output) in tx.outputs.iter().enumerate() {
                if old_output.recipient == spender_pub && self.utxo_set.get(&Outpoint(tx.txid, i as u16)).is_none() {
                    spendable.push(old_output);
                    balance += old_output.amount;
                    if balance >= amount { break; }
                }
            }

            if balance >= amount { break; }
        }

        if amount > balance {
            return Err(())
        }


        let mut tx = Tx::new();
        let mut total: u64 = 0;

        let mut hasher = Sha3_256::new();
        //send the remainder of the last tx back to the user
        let split_last = balance > amount;

        for prev_output in spendable.iter().take(spendable.len()-1) {
            hasher.update(prev_output.as_bytes());
        }

        if split_last {
            let recipient_out = TxOutput {
                spender: TxPredicate::Pubkey(spender_pub),
                amount: amount-(balance-spendable.last().unwrap().amount),
                recipient: recipient_pub,
            };

            //sent back to the recipient
            let remainder_out = TxOutput {
                spender: TxPredicate::Pubkey(spender_pub),
                amount: balance - amount,
                recipient: recipient_pub,
            };

            hasher.update(recipient_out.as_bytes());
            hasher.update(remainder_out.as_bytes());
            tx.outputs.push(recipient_out);
            tx.outputs.push(remainder_out);
        } else {
            //don't feel like moving it lol
            tx.outputs.push((*spendable.last().unwrap()).clone());
        }

        hasher.update(amount.to_be_bytes());
        tx.txid = hasher.finalize().into();

        for (i, prev_output) in tx.outputs.iter().enumerate() {
            self.utxo_set.insert(Outpoint(tx.txid, i as u16), (*prev_output).clone());
        }
        //have to split last output into two outputs
        //if the amounts dont match

        Ok(tx)
    }

}
