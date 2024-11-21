#![allow(unused_imports)]
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use std::fmt;

use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey}, elliptic_curve::sec1::ToEncodedPoint, PublicKey, SecretKey
};
use sha3::*;
use std::{hash::Hash, io::Read};

//choose a better type later
pub type Txid = [u8; 32];
pub const EMPTY_TXID: Txid = [0; 32];

#[derive(Clone)]
pub struct TxOutput {
    //Use Predicate instead of just key to support
    //scripting in the future
    pub spender: TxPredicate,
    //amount is one one millionth of a coin (1 / 10^6)
    pub amount: u64,
    pub recipient: PublicKey,
}


//the second parameter u16 is just an index into the transaction outputs
//TxOutputs are converted into Outpoints so the key doesn't have
//to be stored
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Outpoint(pub Txid, pub u16);

#[derive(Clone)]
pub struct TxInput {
    //point of the signature here
    //is so you can verify that the spender
    //signed this transaction

    //signature of the outpoint
    //which contains the PREVIOUS Txid followed by u16
    pub signature: Signature,
    pub prev_out: Outpoint,
}


#[derive(Clone)]
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

#[derive(Clone)]
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
        bytes.extend_from_slice(&self.recipient.to_sec1_bytes());
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
        //let blank_sig =
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

 //thank you claude!
 impl Serialize for TxInput {
     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
     where
         S: Serializer,
     {
         use serde::ser::SerializeStruct;
         let mut state = serializer.serialize_struct("TxInput", 2)?;
         state.serialize_field("signature", &self.signature.to_bytes().to_vec())?;
         state.serialize_field("prev_out", &self.prev_out.as_bytes())?;
         state.end()
     }
 }

 impl<'de> Deserialize<'de> for TxInput {
     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
     where
         D: Deserializer<'de>,
     {
         #[derive(Deserialize)]
         struct TxInputHelper {
             signature: Vec<u8>,
             prev_out: Vec<u8>,
         }

         let helper = TxInputHelper::deserialize(deserializer)?;

         let signature = Signature::from_slice(&helper.signature)
             .map_err(de::Error::custom)?;

         let mut txid = [0u8; 32];
         txid.copy_from_slice(&helper.prev_out[..32]);
         let index = u16::from_be_bytes([helper.prev_out[32], helper.prev_out[33]]);

         Ok(TxInput {
             signature,
             prev_out: Outpoint(txid, index),
         })
     }
 }



 impl Serialize for TxOutput {
     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
     where
         S: Serializer,
     {
         use serde::ser::SerializeStruct;
         let mut state = serializer.serialize_struct("TxOutput", 3)?;
         match &self.spender {
             TxPredicate::Pubkey(key) => {
                 state.serialize_field("spender", &key.to_encoded_point(false).as_bytes().to_vec())?;
             }
         }
         state.serialize_field("amount", &self.amount)?;
         state.serialize_field("recipient", &self.recipient.to_encoded_point(false).as_bytes().to_vec())?;
         state.end()
     }
 }

 impl<'de> Deserialize<'de> for TxOutput {
     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
     where
         D: Deserializer<'de>,
     {
         #[derive(Deserialize)]
         struct TxOutputHelper {
             spender: Vec<u8>,
             amount: u64,
             recipient: Vec<u8>,
         }

         let helper = TxOutputHelper::deserialize(deserializer)?;

         let spender_key = PublicKey::from_sec1_bytes(&helper.spender)
             .map_err(de::Error::custom)?;

         let recipient = PublicKey::from_sec1_bytes(&helper.recipient)
             .map_err(de::Error::custom)?;

         Ok(TxOutput {
             spender: TxPredicate::Pubkey(spender_key),
             amount: helper.amount,
             recipient,
         })
     }
 }

 impl Serialize for Tx {
     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
     where
         S: Serializer,
     {
         use serde::ser::SerializeStruct;
         let mut state = serializer.serialize_struct("Tx", 4)?;
         state.serialize_field("inputs", &self.inputs)?;
         state.serialize_field("outputs", &self.outputs)?;
         state.serialize_field("txid", &self.txid.to_vec())?;
         state.serialize_field("signature", &self.signature.to_bytes().to_vec())?;
         state.end()
     }
 }

 impl<'de> Deserialize<'de> for Tx {
     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
     where
         D: Deserializer<'de>,
     {
         #[derive(Deserialize)]
         struct TxHelper {
             inputs: Vec<TxInput>,
             outputs: Vec<TxOutput>,
             txid: Vec<u8>,
             signature: Vec<u8>,
         }

         let helper = TxHelper::deserialize(deserializer)?;

         let mut txid = [0u8; 32];
         txid.copy_from_slice(&helper.txid);

         let signature = Signature::from_slice(&helper.signature)
             .map_err(de::Error::custom)?;

         Ok(Tx {
             inputs: helper.inputs,
             outputs: helper.outputs,
             txid,
             signature,
         })
     }
 }
