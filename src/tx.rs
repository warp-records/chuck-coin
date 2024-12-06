#![allow(unused_imports)]

use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey}, elliptic_curve::sec1::ToEncodedPoint, PublicKey, SecretKey
};
use sha3::*;
use serde::*;
use std::{hash::Hash, io::Read};

//choose a better type later
pub type Txid = [u8; 32];
pub const EMPTY_TXID: Txid = [0; 32];

#[derive(Serialize, Deserialize, Clone)]
pub struct TxOutput {
    //Use Predicate instead of just key to support
    //scripting in the future
    pub spender: TxPredicate,
    //amount is one millionth of a coin (1 / 10^6)
    pub amount: u64,
    pub recipient: PublicKey,
}


//the second parameter u16 is just an index into the transaction outputs
//TxOutputs are converted into Outpoints so the key doesn't have
//to be stored
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Outpoint(pub Txid, pub u16);

#[derive(Serialize, Deserialize, Clone)]
pub struct TxInput {
    //point of the signature here
    //is so you can verify that the spender
    //signed this transaction

    //signature of the outpoint
    //which contains the PREVIOUS Txid followed by u16
    pub signature: Signature,
    pub prev_out: Outpoint,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Tx {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    //hash of:
    //tx.inputs.as_bytes()
    //tx.outputs.as_bytes()
    pub txid: Txid,
    //signature of txid, which is the hash
    pub signature: Signature,
}

//let mut utxo_set: HashMap<Outpoint, Tx> = HashMap::new();

#[derive(Serialize, Deserialize, Clone)]
pub enum TxPredicate {
    Pubkey(PublicKey)
}

impl TxPredicate {
    pub fn unwrap_key(&self) -> &PublicKey {
        match &self {
            TxPredicate::Pubkey(key) => &key,
        }
    }
}

impl TxInput {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        //bytes.extend_from_slice(&self.signature.to_bytes());
        bytes.extend_from_slice(&self.prev_out.0);
        bytes.extend_from_slice(&self.prev_out.1.to_be_bytes());
        bytes
    }
}

impl TxOutput {

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match &self.spender {
            TxPredicate::Pubkey(key) => {
                bytes.extend_from_slice(&key.to_encoded_point(false).as_bytes());
            }
        }
        bytes.extend_from_slice(&self.amount.to_be_bytes());
        bytes
    }

}

impl Hash for TxOutput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl Tx {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for input in &self.inputs {
            bytes.extend_from_slice(&input.as_bytes());
        }
        for output in &self.outputs {
            bytes.extend_from_slice(&output.as_bytes());
        }
        bytes.extend_from_slice(&self.signature.to_bytes());
        bytes.extend_from_slice(&self.txid);
        bytes
    }

    //sign only the inputs and outputs
    pub fn get_txid(&self) -> Txid {
        let mut bytes = Vec::new();
        for input in &self.inputs {
            bytes.extend_from_slice(&input.as_bytes());
        }
        for output in &self.outputs {
            bytes.extend_from_slice(&output.as_bytes());
        }

        let mut hasher = Sha3_256::new();
        hasher.update(bytes);

        hasher.finalize().into()
    }

    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            txid: EMPTY_TXID,
            signature: Signature::from_slice(&[128u8; 64]).unwrap(),
        }
    }
}

impl Hash for Tx {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl Outpoint {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0);
        bytes.extend_from_slice(&self.1.to_be_bytes());
        bytes
    }
}



//thanks
/*
fn verify_tx(tx: &Tx, utxoset: &HashMap<Outpoint, UtxoData>) -> bool {
  let mut in_val = 0;
  for (i, inp) in tx.inputs.iter().enumerate() {
    let Some(utxo_data) = utxoset.get(inp.prevout()) else {
      return false;
    }
    verify_witness(inp.witness(), utxo_data.script_pubkey(), tx, i);
    in_val += utxo_data.value();
  }

  let mut out_val = 0;
  for outp in tx.outputs() {
    out_val += outp.value();
  }

  in_val >= out_val
}
 */
