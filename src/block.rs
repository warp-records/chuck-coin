#![allow(unused_imports)]

use serde::*;
use std::fs;
use rand::prelude::*;
use bitvec::prelude::*;
use std::hash::Hash;
use crate::tx::*;
use crate::user::*;
use std::collections::HashMap;
use ethnum::*;
use serde::{Deserialize, Serialize};
use sha3::*;
use k256::{
    Secp256k1,
    ecdsa::{SigningKey, signature::Verifier, VerifyingKey, Signature, signature::Signer},
    SecretKey,
    elliptic_curve::{ sec1::*, PublicKey},
};

pub type BlockHash = [u8; 32];
const BLANK_BLOCK_HASH: [u8; 32] = [0; 32];

#[derive(Serialize, Deserialize)]
//will prune blocks later
pub struct State {
    pub blocks: Vec<Block>,
    //old_utxo_set is needed so we can create multiple new transactions
    //without adding them to the block yet, without having to verfiy
    //all the old blocks again
    //verify_block to use
    #[serde(skip)]
    pub old_utxo_set: HashMap<Outpoint, TxOutput>,
    #[serde(skip)]
    pub utxo_set: HashMap<Outpoint, TxOutput>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    //apparently the utxoset isn't supposed to belong
    //to a particular block, look into this
    pub version: u64,
    pub prev_hash: BlockHash,
    pub nonce: u64,
    pub txs: Vec<Tx>,
}

#[derive(Debug)]
pub enum BlockErr {
    //erroneous nonce
    Nonce(u64),
    //erroneous signature
    Sig(Signature),
    //input amount, output amount
    Overspend(u64, u64),
    //erroneous start supply
    Supply(u64),
    FalseInput,
    //given, expected
    PrevHash(BlockHash, BlockHash),
    GenesisBlock,
}

//should probably rework this interface for clarity
impl State {
    //TODO:
    //verify supply

    //creates a State with a single block which:
    //- has one block
    //- with one transaction
    //- and one transaction output
    //- with an amount of START_SUPPLY
    //- sent and recieved by the keyholder of private_key.txt
    //- and a signature of an empty slice
    //- with an empty TXID
    pub fn verify_all_blocks(&self) -> Result<HashMap<Outpoint, TxOutput>, BlockErr> {
        let mut utxo_set = HashMap::new();
        let mut block_iter = self.blocks.iter();

        let mut prev_block = block_iter.next().unwrap();
        let root_tx = prev_block.txs[0].clone();
        let my_verifying_key: VerifyingKey = vk_from_encoded_str(
            "04B0B5D59947A744C8ED5032F8B5EC77F56BFF09A724466397E82\
            61ABE15BB1F1EC90871F5034A7B2BBF43F33C99225EF70C6F463B3\
            93973C55E85382F90F2935E"
        ).into();

        if prev_block.txs.len() > 1 || root_tx.outputs.len() > 1 ||
            !my_verifying_key.verify(&prev_block.txs[0].txid, &prev_block.txs[0].signature).is_ok() {

            return Err(BlockErr::GenesisBlock);
        }

        utxo_set.insert(Outpoint(root_tx.txid, 0), root_tx.outputs[0].clone());

        while let Some(block) = block_iter.next() {
            utxo_set = self.verify_block(&utxo_set, prev_block, block)?;
            prev_block = block;
        }

        Ok(utxo_set)
    }

    pub fn with_genesis_block() -> State {
        let mut utxo_set = HashMap::new();
        let block = Block::genesis_block();
        utxo_set.insert(Outpoint(block.txs[0].txid, 0), block.txs[0].outputs[0].clone());

        State {
            blocks: vec![block],
            old_utxo_set: utxo_set.clone(),
            utxo_set,
        }
    }

    //can't use this syntax for some reason
    //type UtxoSet = HashMap<Outpoint, TxOutput>;
    pub fn verify_block(&self, old_utxo_set: &HashMap<Outpoint, TxOutput>, prev_block: &Block, block: &Block) -> Result<HashMap<Outpoint, TxOutput>, BlockErr> {
            let mut hasher = Sha3_256::new();
            let mut utxo_set = old_utxo_set.clone();
            //keep track of balances
            let mut input_total: u64 = 0;
            let mut output_total: u64 = 0;

            for tx in &block.txs {
                //TODO: verify tx signature

                for input in tx.inputs.iter() {
                    //check that all inputs being used exited previously
                    let Some(prev_out) = utxo_set.get(&input.prev_out) else {
                        //uh oh...
                        return Err(BlockErr::FalseInput);
                    };


                    //outpoint must be outpoint of prev_out
                    //make sure the owner authorized the transaction
                    if !Block::verify_sig(input.signature, &TxPredicate::Pubkey(prev_out.recipient), &input.prev_out) {
                        //nice try hackers
                        return Err(BlockErr::Sig(input.signature));
                    }

                    //check last block hash
                    hasher.update(&prev_block.as_bytes_no_nonce());
                    let prev_hash = prev_block.get_hash();

                    if prev_hash != block.prev_hash {
                        return Err(BlockErr::PrevHash(block.prev_hash, prev_hash));
                    }


                    //pretty sure we DON'T have to check
                    //the amount from each individual spender
                    input_total += prev_out.amount;
                    //whoops, forgot to add this lol
                }

                for (i, output) in tx.outputs.iter().enumerate() {
                    output_total += output.amount;
                    utxo_set.insert(Outpoint(tx.txid, i as u16), output.clone());
                }

                for input in tx.inputs.iter() {
                    utxo_set.remove(&input.prev_out);
                }

                if output_total > input_total {
                    return Err(BlockErr::Overspend(input_total, output_total));
                }
            }

            if !block.verify_work() {
                return Err(BlockErr::Nonce(block.nonce));
            }

            Ok(utxo_set)
        }

        pub fn add_block_if_valid(&mut self, block: Block) -> Result<(), BlockErr> {
            let new_utxo_set = self.verify_block(&self.old_utxo_set, &self.blocks.last().unwrap(), &block)?;
            self.blocks.push(block);
            self.utxo_set = new_utxo_set.clone();
            self.old_utxo_set = new_utxo_set;

            Ok(())
        }

    pub fn verify_all_and_update(&mut self) -> Result<(), BlockErr>{
        self.utxo_set = self.verify_all_blocks()?;
        self.old_utxo_set = self.utxo_set.clone();
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
    pub const WORK_DIFFICULTY: u64 = u64::max_value() / 1_000;
    //one pizza is one one millionth of a coin, or 1/10^6
    pub const START_SUPPLY: u64 = 69 * 1_000_000;
    pub const TOTAL_SUPPLY: u64 = 420 * 1_000_000;

    pub fn new() -> Self {
        Self {
            version: 0,
            prev_hash: BLANK_BLOCK_HASH,
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
    pub fn transact(&mut self, utxo_set: &mut HashMap<Outpoint, TxOutput>, spender_priv: &SigningKey, recipient_pub: &VerifyingKey, amount: u64) -> Result<&Tx, ()> {
        let spender_pub = PublicKey::from(VerifyingKey::from(spender_priv.clone()));
        let recipient_pub = PublicKey::from(VerifyingKey::from(recipient_pub.clone()));

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

        for output in spendable.iter().take(spendable.len() - 1) {
            let mut new_output = output.clone();
            new_output.spender = TxPredicate::Pubkey(spender_pub.clone());
            new_output.recipient = recipient_pub.clone();

            new_tx.outputs.push(new_output);
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
                recipient: spender_pub,
            };

            new_tx.outputs.push(recipient_out);
            new_tx.outputs.push(remainder_out);
        } else {
            //don't feel like moving it lol
            let output = TxOutput {
                spender: TxPredicate::Pubkey(spender_pub),
                amount: spendable.last().unwrap().amount,
                recipient: recipient_pub,
            };
            new_tx.outputs.push(output);
        }

        new_tx.txid = new_tx.get_txid();
        new_tx.signature = spender_priv.sign(&new_tx.txid);

        for input in new_tx.inputs.iter() {
            utxo_set.remove(&input.prev_out).expect("Didn't find prev out");
        }
        //critical part
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
        let block_hash = self.get_hash();

        hasher.update(block_hash);
        hasher.update(self.nonce.to_le_bytes());
        let work_hash = hasher.finalize();
        let work_hash_64 = u64::from_le_bytes(work_hash[0..8].try_into().unwrap());

        work_hash_64 <= Self::WORK_DIFFICULTY
    }

    pub fn mine(&self) -> u64 {
        let mut rng = rand::thread_rng();
        //start at a random spot so not all
        //nodes are mining from the same spot
        let mut gold: u64 = rng.gen_range(0..Self::WORK_DIFFICULTY);
        let mut hasher = Sha3_256::new();

        let block_hash = self.get_hash();
        loop {
            hasher.update(block_hash);
            hasher.update(gold.to_le_bytes());
            let work_hash = hasher.finalize_reset();
            let work_hash_64 = u64::from_le_bytes(work_hash[0..8].try_into().unwrap());

            if work_hash_64 <= Self::WORK_DIFFICULTY {
                return gold;
            }

            gold += 1;
            if gold > Self::WORK_DIFFICULTY {
                gold = 0;
            }
        }
    }

    pub fn genesis_block() -> Self {
        let mut utxo_set = HashMap::new();
        let my_verifying_key: VerifyingKey = vk_from_encoded_str("04B0B5D59947A744C8ED5032F8B5EC77F56BFF09A724466397E8261ABE15BB1F1EC90871F5034A7B2BBF43F33C99225EF70C6F463B393973C55E85382F90F2935E").into();

        let mut block = Block {
            version: 0,
            prev_hash: BLANK_BLOCK_HASH,
            nonce: 0,
            txs: Vec::new(),
        };

        let public_key = PublicKey::from(my_verifying_key);

        let root_output = TxOutput {
            amount: Block::START_SUPPLY,
            spender: TxPredicate::Pubkey(public_key),
            //I'M RICH
            recipient: public_key,
        };

        let my_signature = "fc839fd7d15231a66be4840c1fe916a8f3963367b69f099\
            7c032839b5a1533da2859e00ab6e842a4bbce351ca435a281913c58638abe61\
            5ff6887ed9a492b9a4";
        let my_signature = Signature::from_slice(&(hex::decode(my_signature).unwrap())).unwrap();

        let mut root_tx = Tx {
            inputs: Vec::new(),
            outputs: vec![root_output.clone()],
            txid: EMPTY_TXID,
            signature: my_signature,
        };
        root_tx.txid = root_tx.get_txid();

        utxo_set.insert(Outpoint(root_tx.txid, 0), root_output.clone());

        block.txs.push(root_tx);
        block
    }
}
//

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
        bytes.extend_from_slice(&self.prev_hash);
        //bytes.extend_from_slice(&self.nonce.to_be_bytes());
        // Convert txs to bytes
        for tx in &self.txs {
            bytes.extend(tx.as_bytes());
        }
        // Convert utxo_set to bytes

        bytes
    }

    pub fn get_hash(&self) -> BlockHash {
        let mut hasher = Sha3_256::new();

        hasher.update(&self.as_bytes_no_nonce());
        let block_hash = hasher.finalize_reset();
        block_hash[0..32].try_into().unwrap()
    }
}
