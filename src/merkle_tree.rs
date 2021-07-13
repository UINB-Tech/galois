use crate::{
    core::{Symbol, Currency, OrderId, Amount, Price},
};

use crate::orderbook::{AskOrBid};

use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore,
    error::Error, MerkleProof,
    SparseMerkleTree, traits::Value, H256,
};
use blake2b_rs::{Blake2b, Blake2bBuilder};
use sha2::{Sha256, Sha512, Digest};
use rust_decimal::Decimal;
use chrono::format::Pad::Zero;

#[derive(Clone)]
pub struct TapeValue { size: Amount, best: Price }

impl Value for TapeValue {
    fn to_h256(&self) -> H256 {
        if self.size == Decimal::ZERO {
            return H256::zero();
        }

        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        let size = self.size.normalize().to_string();
        let size = size.as_bytes();
        hasher.update(size);
        let best = self.best.normalize().to_string();
        let best = best.as_bytes();
        hasher.update(best);
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        TapeValue { size: Default::default(), best: Default::default() }
    }
}

#[derive(Clone)]
pub struct OrderValue { owner: H256, amount: Amount, price: Price, ask_or_bid: u32 }

impl Value for OrderValue {
    fn to_h256(&self) -> H256 {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(self.owner.as_slice());
        let amount = self.amount.normalize().to_string();
        let amount = amount.as_bytes();
        hasher.update(amount);
        let price = self.price.normalize().to_string();
        let price = price.as_bytes();
        hasher.update(price);
        let ot: u32 = self.ask_or_bid.into();
        let t = ot.to_ne_bytes();
        hasher.update(&t);
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        OrderValue {
            owner: H256::zero(),
            amount: Default::default(),
            price: Default::default(),
            ask_or_bid: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct AccountValue { tradable: Decimal, frozen: Decimal }

impl Value for AccountValue {
    fn to_h256(&self) -> H256 {
        if self.tradable == Decimal::ZERO && self.frozen == Decimal::ZERO {
            return H256::zero();
        }

        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();

        let tradable = self.tradable.normalize().to_string();
        let tradable = tradable.as_bytes();
        hasher.update(tradable);
        let frozen = self.frozen.normalize().to_string();
        let frozen = frozen.as_bytes();
        hasher.update(frozen);
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        AccountValue { tradable: Default::default(), frozen: Default::default() }
    }
}

const BLAKE2B_KEY: &[u8] = &[];
const BLAKE2B_LEN: usize = 32;
const PERSONALIZATION: &[u8] = b"sparsemerkletree";

// helper function
fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(BLAKE2B_LEN)
        .personal(PERSONALIZATION)
        .key(BLAKE2B_KEY)
        .build()
}

fn gen_symbol_key(symbol: Symbol, ask_or_bid: AskOrBid) -> H256 {
    let mut buf = [0u8; 32];
    let mut hasher = new_blake2b();
    let b = symbol.0.to_ne_bytes();
    hasher.update(&b);
    let q = symbol.1.to_ne_bytes();
    hasher.update(&q);
    hasher.finalize(&mut buf);
    let symbol_hash: H256 = buf.into();

    hasher = new_blake2b();
    hasher.update(symbol_hash.as_slice());
    let ot: u32 = ask_or_bid.into();
    let ot = ot + 2;
    let t = ot.to_ne_bytes();
    hasher.update(&t);
    hasher.finalize(&mut buf);
    buf.into()
}

fn gen_account_key(address: H256, currency: Currency) -> H256 {
    let mut buf = [0u8; 32];
    let mut hasher = new_blake2b();
    hasher.update(address.as_slice());
    let c = currency.to_ne_bytes();
    hasher.update(&c);
    hasher.finalize(&mut buf);
    buf.into()
}

fn gen_order_id_key(id: OrderId) -> H256 {
    let mut buf = [0u8; 32];
    let mut hasher = new_blake2b();
    let o = id.to_ne_bytes();
    hasher.update(&o);
    hasher.finalize(&mut buf);
    buf.into()
}
