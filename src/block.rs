#![allow(unused_imports)]

use rand::prelude::*;
use bitvec::prelude::*;
use std::hash::Hash;
use crate::tx::*;
use std::collections::HashMap;
use ethnum::*;
//use ethnum::U256::trailing_zeros;
use serde::{Deserialize, Serialize};
use sha3::*;
use k256::{
    ecdsa::{SigningKey, signature::Verifier, VerifyingKey, Signature, signature::Signer},
    SecretKey,
    PublicKey,
};

type BLOCK_HASH = [u8; 32];
const BLANK_BLOCK_HASH: [u8; 32] = [0; 32];

//#[derive(Serialize, Deserialize, Debug)]
//will prune blocks later
pub struct State {
    blocks: Vec<Block>,
}

//#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    //apparently the utxoset isn't supposed to belong
    //to a particular block, look into this
    pub utxo_set: HashMap<Outpoint, TxOutput>,
    pub prev_hash: u64,
    pub nonce: u64,
    pub txs: Vec<Tx>,
}

//lol XD
//const TITTIES = 7177135;
//const BOOBIES = 8008135;
//consider using these as currency limits
//or something in the project

//this shit is hard
impl Block {
    const WORK_DIFFICULTY: u64 = u64::max_value()/100000;
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

        self.verify_work()
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

pub fn verify_work(&self) -> bool {

        let mut hasher = Sha3_256::new();
        hasher.update(self.as_bytes_no_nonce());

        let block_hash = hasher.finalize_reset();

        hasher.update(block_hash);
        hasher.update(self.nonce.to_le_bytes());
        let work_hash = hasher.finalize();
        let work_hash_64 = u64::from_be_bytes(work_hash[0..8].try_into().unwrap());

        work_hash_64 <= Self::WORK_DIFFICULTY
    }

    //pub fn block_work(hash: BLOCK_HASH) -> u64 {
   //     const MAX_NONCE: u64 = u64::MAX;
   //     let hash_64 = u64::from_be_bytes(hash[0..8].try_into().unwrap());
   //     MAX_NONCE - hash_64
   // }

    pub fn mine(&self) -> u64 {
        let mut rng = rand::thread_rng();
        let mut gold: u64 = 0;//rng.gen_range(0..Self::WORK_DIFFICULTY);
        let mut hasher = Sha3_256::new();

        let block_hash = hasher.finalize_reset();
        loop {
            hasher.update(block_hash);
            hasher.update(gold.to_le_bytes());
            let work_hash = hasher.finalize_reset();
            let work_hash_64 = u64::from_be_bytes(work_hash[0..8].try_into().unwrap());

            if work_hash_64 <= Self::WORK_DIFFICULTY {
                return gold;
            }

            gold += 1;
            if gold > Self::WORK_DIFFICULTY {
                gold = 0;
            }
        }
    }
}

impl Hash for Block {
    //DONT hash nonce
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Skip utxo_set since it's not hashable
        self.prev_hash.hash(state);
        self.txs.hash(state);
        for (outpoint, output) in &self.utxo_set {
            outpoint.hash(state);
            output.hash(state);
        }
    }
}

impl Block {
    pub fn as_bytes_no_nonce(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Convert prev_hash to bytes
        bytes.extend_from_slice(&self.prev_hash.to_be_bytes());
        //bytes.extend_from_slice(&self.nonce.to_be_bytes());
        // Convert txs to bytes
        for tx in &self.txs {
            bytes.extend(tx.as_bytes());
        }
        // Convert utxo_set to bytes
        //this may not work
        //for (outpoint, output) in &self.utxo_set {
        //    bytes.extend(outpoint.as_bytes());
        //    bytes.extend(output.as_bytes());
       //}

        bytes
    }
}
