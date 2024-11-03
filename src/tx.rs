use k256::PublicKey;
use rust_decimal::Decimal;
use std::hash::Hash;

pub struct TxOutput {
    spender: PublicKey,
    amount: Decimal,
    txid: u16,
}
