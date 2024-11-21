#![allow(unused_imports)]

use std::fs;
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

type BlockHash = [u8; 32];
const BLANK_BLOCK_HASH: [u8; 32] = [0; 32];
const BLANK_TXID: [u8; 32] = [0; 32];

//#[derive(Serialize, Deserialize, Debug)]
//will prune blocks later
pub struct State {
    pub blocks: Vec<Block>,
    pub utxo_set: HashMap<Outpoint, TxOutput>
}

//#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    //apparently the utxoset isn't supposed to belong
    //to a particular block, look into this
    pub version: u64,
    pub prev_hash: u64,
    pub nonce: u64,
    pub txs: Vec<Tx>,
}

pub enum BlockErr {
    //erroneous nonce
    Nonce(u64),
    //erroneous signature
    Sig(Signature),
    //input amount, output amount
    Overspend(u64, u64),
    //erroneous start supply
    Supply(u64),
    DoubleSpend,
}

impl State {
    //TODO:
    //verify supply
    //verify prevhash

    //creates a State with a single block which:
    //- has one block
    //- with one transaction
    //- and one transaction output
    //- with an amount of START_SUPPLY
    //- sent and recieved by the keyholder of private_key.txt
    //- and a signature of an empty slice
    //- with an empty TXID
    pub fn verify_all_blocks(&self) -> Result<(), BlockErr> {
        //let mut utxo_set = HashMap::<Outpoint, TxOutput>::new();
        let mut utxo_set = HashMap::new();
        let mut block_iter = self.blocks.iter();

        let mut prev_block = block_iter.next().unwrap();
        let root_tx = prev_block.txs[0].clone();
        if prev_block.txs.len() > 1 || root_tx.outputs.len() > 1 {
            panic!("Expected a single root transaction with a single txo");
        }

        utxo_set.insert(Outpoint(root_tx.txid, 0), root_tx.outputs[0].clone());

        while let Some(block) = block_iter.next() {
            //keep track of balances
            let mut input_total: u64 = 0;
            let mut output_total: u64 = 0;

            //verify hashes
            for tx in &block.txs {
                let txid = tx.get_txid();

                for (i, input) in tx.inputs.iter().enumerate() {
                    //check that all inputs being used exited previously
                    let Some(prev_out) = utxo_set.get(&input.prev_out) else {
                        //uh oh...
                        return Err(BlockErr::DoubleSpend);
                    };


                    //outpoint must be outpoint of prev_out
                    if !Block::verify_sig(input.signature, &prev_out.spender, &input.prev_out) {
                        //nice try hackers
                        return Err(BlockErr::Sig(input.signature));
                    }

                    //pretty sure we DON'T have to check
                    //the amount from each individual spender
                    input_total += prev_out.amount;
                }

                for (i, output) in tx.outputs.iter().enumerate() {
                    output_total += output.amount;
                    utxo_set.insert(Outpoint(tx.txid, i as u16), output.clone());
                }

                if output_total > input_total {
                    return Err(BlockErr::Overspend(input_total, output_total));
                }
            }

            if !block.verify_work() {
                return Err(BlockErr::Nonce(block.nonce));
            }
        }

        Ok(())
    }
}

//lol XD
//const TITTIES = 7177135;
//const BOOBIES = 8008135;
//consider using these as currency limits
//or something in the project

//this shit is hard
impl Block {
    //This is all my i7 can do quickly ToT
    //temporarily make it really easy for testing
    pub const WORK_DIFFICULTY: u64 = u64::max_value()/1_000;
    //one pizza is one one millionth of a coin, or 1/10^6
    pub const START_SUPPLY: u64 = 69 * 1_000_000;
    pub const TOTAL_SUPPLY: u64 = 420 * 1_000_000;

    pub fn new() -> Self {
        Self {
                version: 0,
                prev_hash: 0,
                nonce: 0,
                txs: Vec::new(),
        }
    }

    //check that signature equals the hash of tall transactions
    //and the transaction index combined, all signed by the spender
    pub fn verify_sig(sig: Signature, predicate: &TxPredicate, prev_out: &Outpoint) -> bool {

        let verifying_key = VerifyingKey::from(predicate.unwrap_key());
        verifying_key.verify(&prev_out.as_bytes(), &sig).is_ok()
    }

    //must be executed on the spenders hardware
    //since spender_priv is passed as an arugment
    pub fn transact(&mut self, utxo_set: &mut HashMap<Outpoint, TxOutput>, spender_priv: SigningKey, recipient_pub: PublicKey, amount: u64) -> Result<&Tx, ()> {
        let spender_pub: PublicKey = VerifyingKey::from(spender_priv.clone()).into();

        let mut new_tx = Tx::new();

        let mut balance: u64 = 0;
        //I hope I'm doing this right lol
        let mut spendable: Vec<TxOutput> = Vec::new();
        for outpoint in utxo_set.keys() {
            let prev_out = utxo_set.get(outpoint).unwrap();
            if prev_out.recipient == spender_pub {
                new_tx.inputs.push(TxInput {
                    signature: spender_priv.sign(&outpoint.as_bytes()),
                    prev_out: outpoint.clone(),
                });

                spendable.push(prev_out.clone());
                balance += prev_out.amount;
                if balance >= amount { break; }
            }
        }


        if amount > balance {
            return Err(())
        }

        //send the remainder of the last tx back to the user
        let split_last = balance > amount;

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
                recipient: spender_pub,
            };

            new_tx.outputs.push(recipient_out);
            new_tx.outputs.push(remainder_out);
        } else {
            //don't feel like moving it lol
            new_tx.outputs.push((*spendable.last().unwrap()).clone());
        }

        new_tx.txid = new_tx.get_txid();
        new_tx.signature = spender_priv.sign(&new_tx.txid);

        for input in new_tx.inputs.iter() {
            utxo_set.remove(&input.prev_out);
        }
        for (i, output) in new_tx.outputs.iter().enumerate() {
            utxo_set.insert(Outpoint(new_tx.txid, i as u16), (*output).clone());
        }
        //have to split last output into two outputs
        //if the amounts dont match
        self.txs.push(new_tx);

        Ok(&self.txs.last().unwrap())
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
        //start at a random spot so not all
        //nodes are mining from the same spot
        let mut gold: u64 = rng.gen_range(0..Self::WORK_DIFFICULTY);
        let mut hasher = Sha3_256::new();

        hasher.update(&self.as_bytes_no_nonce());
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
//

/*
impl State {
    fn verify_blockchain() -> bool {

    }
}
 */
impl Hash for Block {
    //DONT hash nonce
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Skip utxo_set since it's not hashable
        self.version.hash(state);
        self.prev_hash.hash(state);
        self.txs.hash(state);
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

impl State {
    pub fn with_inital_block() -> Self {
        let mut utxo_set = HashMap::new();
        let priv_key_str = fs::read_to_string("private_key.txt").expect("Expected private Secp256k1 key in file \"private_key.txt\"");
        let signing_key = SigningKey::from_bytes(hex::decode(priv_key_str).unwrap().as_slice().into()).unwrap();
        let verifying_key = VerifyingKey::from(signing_key.clone());

        let mut block = Block {
            version: 0,
            prev_hash: 0,
            nonce: 0,
            txs: Vec::new(),
        };

        let MYYY_SPEECIAAALLL_KEEEYYY_FUCKYEAH = PublicKey::from(verifying_key);

        let root_output = TxOutput {
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

        root_tx.txid = root_tx.get_txid();
        root_tx.signature = signing_key.sign(&root_tx.txid);
        root_tx.outputs.push(root_output.clone());

        utxo_set.insert(Outpoint(root_tx.txid, 0), root_output.clone());

        block.txs.push(root_tx);
        Self {
            blocks: vec![block],
            utxo_set,
        }
    }
}
