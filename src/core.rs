// Copyright 2021 UINB Technologies Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{assets::Account, orderbook::OrderBook};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
};

pub type Base = u32;
pub type Quote = u32;
pub type Price = Decimal;
pub type Amount = Decimal;
pub type Vol = Decimal;
pub type Currency = u32;
pub type UserId = u64;
pub type Symbol = (Base, Quote);
pub type EventId = u64;
pub type OrderId = u64;
pub type Fee = Decimal;
pub type Scale = u32;
pub type Timestamp = u64;

pub type Accounts = HashMap<UserId, HashMap<Currency, Account>>;

pub const SYSTEM: u64 = 0;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Data {
    pub orderbooks: HashMap<Symbol, OrderBook>,
    pub accounts: Accounts,
}

unsafe impl Sync for Data {}

impl Data {
    pub fn new() -> Self {
        Self {
            orderbooks: HashMap::new(),
            accounts: HashMap::new(),
        }
    }

    pub fn from_raw(file: File) -> anyhow::Result<Self> {
        let reader = BufReader::new(file);
        let mut decompress = ZlibDecoder::new(reader);
        Ok(bincode::deserialize_from(&mut decompress)?)
    }

    pub fn into_raw(&self, file: File) -> anyhow::Result<()> {
        let writer = BufWriter::new(file);
        let mut compress = ZlibEncoder::new(writer, Compression::best());
        bincode::serialize_into(&mut compress, &self)?;
        Ok(())
    }
}

#[test]
pub fn test_dump() {
    use crate::orderbook::Order;
    let order = Order::new(0, 0, Decimal::new(1, 0), Decimal::new(1, 0));
    let v = bincode::serialize(&order).unwrap();
    let des: Order = bincode::deserialize(&v).unwrap();
    assert_eq!(des, order);
    let orderbook = OrderBook::new(
        3,
        3,
        Decimal::new(1, 3),
        Decimal::new(1, 3),
        Decimal::new(1, 3),
        Decimal::new(1, 0),
        false,
    );
    let v = bincode::serialize(&orderbook).unwrap();
    let des: OrderBook = bincode::deserialize(&v).unwrap();
    assert_eq!(des, orderbook);
    let mut test = Data::new();
    test.orderbooks.insert(
        (101, 100),
        OrderBook::new(
            3,
            3,
            Decimal::new(1, 3),
            Decimal::new(1, 3),
            Decimal::new(1, 3),
            Decimal::new(1, 0),
            false,
        ),
    );

    let temp_dir = tempdir::TempDir::new(".").unwrap();
    let file_path = temp_dir.path().join("bin.gz");
    let temp_file = File::create(&file_path).unwrap();
    test.into_raw(temp_file).unwrap();

    let de = Data::from_raw(File::open(&file_path).unwrap()).unwrap();
    assert_eq!(test, de);
}
