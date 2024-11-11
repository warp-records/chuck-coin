use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey}, elliptic_curve::sec1::ToEncodedPoint, PublicKey, SecretKey
};
use std::hash::Hash;

//choose a better type later
type Txid = [u8; 32];
const EMPTY_TXID: Txid = [0; 32];

#[derive(Clone)]
pub struct TxOutput {
    //Use Predicate instead of just key to support
    //scripting in the future
    pub spender: TxPredicate,
    //amount is one one millionth of a coin (1 / 10^6)
    pub amount: u64,

    //Txid is Sha3_256 hash of:
    //- all input txs as_bytes()
    //- amount to_be_bytes()
    //- spender to_sec1_bytes()
    //- recipient to_sec1_bytes()
    //in THAT ORDER
    pub recipient: PublicKey,
}


//pretty sure the u16 is just an index into the transaction inputs
//TxOutputs are converted into Outpoints so the key doesn't have
//to be stored
#[derive(Hash, PartialEq, Eq)]
pub struct Outpoint(pub Txid, pub u16);

pub struct TxInput {
    pub signature: Signature,
    pub prev_out: Outpoint,
}


pub struct Tx {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub txid: Txid,
    //hash signed by the spender of:
    //tx.inputs.as_bytes()
    //tx.outputs.as_bytes()
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
            _ => panic!(),
        }
    }
}

impl TxInput {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.signature.to_bytes());
        bytes.extend_from_slice(&self.prev_out.0);
        bytes.extend_from_slice(&self.prev_out.1.to_be_bytes());
        bytes
    }
}

impl TxOutput {

    //used to hash all other data besides txid
    // necessary for creating txid in the first place
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
        bytes
    }

    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            txid: EMPTY_TXID,
            signature: Signature::from_slice(&[0u8; 64]).unwrap(),
        }
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
