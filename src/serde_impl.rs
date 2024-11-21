


//THANK YOU CLAUDE!
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de;
use k256::ecdsa::{Signature, VerifyingKey};
use k256::PublicKey;
use k256::elliptic_curve::sec1::*;
use crate::tx::{TxInput, TxOutput, Tx, Outpoint, TxPredicate};
use crate::block::*;

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Block", 4)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("prev_hash", &self.prev_hash)?;
        state.serialize_field("nonce", &self.nonce)?;
        state.serialize_field("txs", &self.txs)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Block {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BlockHelper {
            version: u64,
            prev_hash: u64,
            nonce: u64,
            txs: Vec<Tx>,
        }

        let helper = BlockHelper::deserialize(deserializer)?;

        Ok(Block {
            version: helper.version,
            prev_hash: helper.prev_hash,
            nonce: helper.nonce,
            txs: helper.txs,
        })
    }
}

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



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use k256::{
        ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
        SecretKey,
    };
    use k256::elliptic_curve::sec1::ToEncodedPoint;


    pub fn keys_from_str(priv_key: &str) -> (SigningKey, VerifyingKey) {
        let signing_key = SigningKey::from_bytes(hex::decode(priv_key).unwrap().as_slice().into()).unwrap();
        let verifying_key = VerifyingKey::from(signing_key.clone());

        (signing_key, verifying_key)
    }


    #[test]
    fn test_tx_input_serde() {
        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        let signature = signing_key.sign(b"test message");
        let outpoint = Outpoint([1u8; 32], 0);

        let tx_input = TxInput {
            signature,
            prev_out: outpoint
        };

        let serialized = serde_json::to_string(&tx_input).unwrap();
        let deserialized: TxInput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(tx_input.prev_out.0, deserialized.prev_out.0);
        assert_eq!(tx_input.prev_out.1, deserialized.prev_out.1);
        assert_eq!(tx_input.signature.to_bytes(), deserialized.signature.to_bytes());
    }


    #[test]
    fn test_tx_output_serde() {
        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        let tx_output = TxOutput {
            spender: TxPredicate::Pubkey(verifying_key.into()),
            amount: 1000,
            recipient: verifying_key.into(), // Using same key for test simplicity
        };

        let serialized = serde_json::to_string(&tx_output).unwrap();
        let deserialized: TxOutput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(tx_output.amount, deserialized.amount);
        assert_eq!(
            tx_output.spender.unwrap_key().to_encoded_point(false).as_bytes(),
            deserialized.spender.unwrap_key().to_encoded_point(false).as_bytes()
        );
        assert_eq!(
            tx_output.recipient.to_encoded_point(false).as_bytes(),
            deserialized.recipient.to_encoded_point(false).as_bytes()
        );
    }

    #[test]
    fn test_tx_serde() {
        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        // Create a test transaction
        let mut tx = Tx::new();

        // Add an input
        let signature = signing_key.sign(b"test message");
        let outpoint = Outpoint([1u8; 32], 0);
        tx.inputs.push(TxInput {
            signature,
            prev_out: outpoint,
        });

        // Add an output
        tx.outputs.push(TxOutput {
            spender: TxPredicate::Pubkey(verifying_key.into()),
            amount: 1000,
            recipient: verifying_key.into(),
        });

        // Set txid and signature
        tx.txid = tx.get_txid();
        tx.signature = signing_key.sign(&tx.txid);

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&tx).unwrap();
        let deserialized: Tx = serde_json::from_str(&serialized).unwrap();

        assert_eq!(tx.txid, deserialized.txid);
        assert_eq!(tx.signature.to_bytes(), deserialized.signature.to_bytes());
        assert_eq!(tx.inputs.len(), deserialized.inputs.len());
        assert_eq!(tx.outputs.len(), deserialized.outputs.len());
    }

    #[test]
    fn test_block_serde() {
        let (signing_key, verifying_key) = keys_from_str(&fs::read_to_string("private_key.txt").unwrap());

        // Create a test block
        let mut block = Block::new();
        block.version = 1;
        block.prev_hash = 12345;
        block.nonce = 67890;

        // Add a transaction
        let mut tx = Tx::new();
        tx.inputs.push(TxInput {
            signature: signing_key.sign(b"test message"),
            prev_out: Outpoint([1u8; 32], 0),
        });
        tx.outputs.push(TxOutput {
            spender: TxPredicate::Pubkey(verifying_key.into()),
            amount: 1000,
            recipient: verifying_key.into(),
        });
        tx.txid = tx.get_txid();
        tx.signature = signing_key.sign(&tx.txid);

        block.txs.push(tx);

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&block).unwrap();
        let deserialized: Block = serde_json::from_str(&serialized).unwrap();

        assert_eq!(block.version, deserialized.version);
        assert_eq!(block.prev_hash, deserialized.prev_hash);
        assert_eq!(block.nonce, deserialized.nonce);
        assert_eq!(block.txs.len(), deserialized.txs.len());
    }
}
